use vecmath::*;
use message::*;
use config::{ControllerMode, Config, GimbalTrackingGain};
use fygimbal;
use fygimbal::protocol::{target, values};
use fygimbal::util::vec2_encoder_sub;
use fygimbal::GimbalPort;
use std::time::{Duration, Instant};

struct GimbalValueState {
    last_requested: Option<Instant>,
    last_update: Option<(Instant, GimbalValueData)>,
}

impl GimbalValueState {
    fn new() -> GimbalValueState {
        GimbalValueState {
            last_requested: None,
            last_update: None,
        }
    }

    fn poll_for_request(&mut self, interval_millis: u64) -> bool {
        let now = Instant::now();
        let needs_request = match self.last_requested {
            None => true,
            Some(timestamp) => now > timestamp + Duration::from_millis(interval_millis),
        };
        if needs_request {
            self.last_requested = Some(now);
            true
        } else {
            false
        }
    }

    fn value_with_max_age(&self, stale_flag: &mut bool, millis: u64) -> i16 {
        let max_age = Duration::from_millis(millis);
        let now = Instant::now();
        match self.last_update {
            None => {
                *stale_flag = true;
                0
            },
            Some((timestamp, ref data)) => {
                if now > timestamp + max_age {
                    *stale_flag = true;
                }
                data.value
            }
        }
    }
}

pub struct GimbalController {
    values: Vec<Vec<GimbalValueState>>,
    yaw_tracking_i: Vec<f32>,
    pitch_tracking_i: Vec<f32>,
    drift_compensation: Vector2<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum RequestType {
    Continuous,
    Infrequent,
}

impl GimbalController {
    pub fn new() -> GimbalController {
        GimbalController {
            values: (0 .. fygimbal::protocol::NUM_VALUES).map(|_| {
                (0 .. fygimbal::protocol::NUM_AXES).map(|_| {
                    GimbalValueState::new()
                 }).collect()
            }).collect(),
            yaw_tracking_i: Vec::new(),
            pitch_tracking_i: Vec::new(),
            drift_compensation: [0.0, 0.0],
        }
    }

    pub fn tick(&mut self, config: &Config, gimbal: &GimbalPort, tracked: &CameraTrackedRegion) -> GimbalControlStatus {
        let mut stale_flag = false;
        let center_cal = self.request_vec2(gimbal, &mut stale_flag, RequestType::Infrequent, values::CENTER_CALIBRATION);
        let raw_angles = self.request_vec2(gimbal, &mut stale_flag, RequestType::Continuous, values::ENCODER_ANGLES);
        let angles = vec2_encoder_sub(raw_angles, center_cal);

        let mut status = GimbalControlStatus {
            angles,
            rates: [0, 0],
            tracking_p_rates: [0.0, 0.0],
            tracking_i_rates: [0.0, 0.0],
            yaw_gain_activations: config.gimbal.yaw_gains.iter().map(|_| 0.0).collect(),
            pitch_gain_activations: config.gimbal.pitch_gains.iter().map(|_| 0.0).collect(),
            drift_compensation: self.drift_compensation,
        };

        if !stale_flag {
            self.tracking_tick(config, tracked, &mut status);
        }

        gimbal.write_control_rates(status.rates);
        status
    }

    pub fn drift_compensation_tracking_update(&mut self, config: &Config, tracked: CameraTrackedRegion) {
        let diff = vec2_sub(rect_center(tracked.previous_rect), rect_center(tracked.rect));
        if vec2_square_len(diff) < config.gimbal.drift_rect_speed_threshold.powi(2) {
            let comp = vec2_mul(diff, config.gimbal.drift_compensation_gain);
            let comp = vec2_add(self.drift_compensation, comp);
            let comp = vec2_clamp_len(comp, config.gimbal.drift_compensation_max);
            self.drift_compensation = comp;
        }
    }

    fn tracking_tick(&mut self, config: &Config, tracked: &CameraTrackedRegion, status: &mut GimbalControlStatus) {
        let border = config.vision.border_rect;
        let left_dist = rect_left(tracked.rect) - rect_left(border);
        let top_dist = rect_top(tracked.rect) - rect_top(border);
        let right_dist = rect_right(border) - rect_right(tracked.rect);
        let bottom_dist = rect_bottom(border) - rect_bottom(tracked.rect);

        let axis = |i_state: &mut Vec<f32>, errs: &mut Vec<f32>, gains: &Vec<GimbalTrackingGain>, lower_dist: f32, upper_dist: f32| {
            // Initialize internal integrator state at init or when number of gains change
            i_state.truncate(gains.len());
            while i_state.len() < gains.len() {
                i_state.push(0.0);
            }

            let mut p = 0.0;
            let mut i = 0.0;
            for index in 0 .. gains.len() {
                let gain = &gains[index];
                let err = (gain.width - lower_dist).max(0.0) - (gain.width - upper_dist).max(0.0);
                errs[index] = err;
                if i_state[index] * err <= 0.0 {
                    // Halt or change directions; clear integral gain accumulator
                    i_state[index] = 0.0;
                }
                i_state[index] += err;
                p += err * gain.p_gain;
                i += i_state[index] * gain.i_gain;
            }
            (p, i)
        };

        let (xp, xi) = axis(&mut self.yaw_tracking_i, &mut status.yaw_gain_activations, &config.gimbal.yaw_gains, left_dist, right_dist);
        let (yp, yi) = axis(&mut self.pitch_tracking_i, &mut status.pitch_gain_activations, &config.gimbal.pitch_gains, top_dist, bottom_dist);
        status.tracking_p_rates = [xp, yp];
        status.tracking_i_rates = [xi, yi];

        let rates = if config.mode == ControllerMode::Halted {
            // Tracking disabled (but limiter and drift compensation work)
            [0.0, 0.0]
        } else {
            vec2_add(status.tracking_i_rates, status.tracking_p_rates)
        };

        let rates = vec2_add(rates, status.drift_compensation);
        let rates = self.limiter(config, status.angles, rates);
        let rates = vec2_clamp_len(rates, config.gimbal.max_rate);
        status.rates = self.dither_rates(rates);
    }

    fn limiter(&self, config: &Config, angles: Vector2<i16>, rates: Vector2<f32>) -> Vector2<f32> {

        let single_edge = |rate: f32, distance: i16, out_dir: i16, slowdown_extent: f32| {
            let outside_distance = distance * out_dir;
            let rate = rate + if outside_distance > 0 {
                // Past the limit; push back inward
                (-distance as f32) * config.gimbal.limiter_gain
            } else {
                0.0
            };

            let outward_limit = if outside_distance >= 0 {
                0.0
            } else {
                (-outside_distance as f32) / slowdown_extent
            };
            let outward_limit = outward_limit.max(0.0).min(1.0) * config.gimbal.max_rate;

            if out_dir > 0 { rate.min(outward_limit) } else { rate.max(-outward_limit) }
        };

        [
            single_edge(single_edge(rates[0],
                angles[0] - config.gimbal.yaw_limits.0, -1, config.gimbal.limiter_slowdown_extent[0]),
                angles[0] - config.gimbal.yaw_limits.1, 1, config.gimbal.limiter_slowdown_extent[0]),

            single_edge(single_edge(rates[1],
                angles[1] - config.gimbal.pitch_limits.0, -1, config.gimbal.limiter_slowdown_extent[1]),
                angles[1] - config.gimbal.pitch_limits.1, 1, config.gimbal.limiter_slowdown_extent[1])
        ]
    }

    fn dither_rates(&self, rates: Vector2<f32>) -> Vector2<i16> {
        let rates = vec2_add(rates, vec2_rand_from_centered_unit_square());
        [rates[0].round() as i16, rates[1].round() as i16]
    }

    fn request(&mut self, gimbal: &GimbalPort, stale_flag: &mut bool, rtype: RequestType, index: u8, target: u8) -> i16 {
        let slot = &mut self.values[index as usize][target as usize];
        match rtype {
            RequestType::Continuous => {
                gimbal.request_continuous(index, target);
                slot.value_with_max_age(stale_flag, 250)
            },
            RequestType::Infrequent => {
                if slot.poll_for_request(1000) {
                    gimbal.request_once(index, target);
                }
                slot.value_with_max_age(stale_flag, 2000)
            }
        }
    }

    fn request_vec2(&mut self, gimbal: &GimbalPort, stale_flag: &mut bool, rtype: RequestType, index: u8) -> Vector2<i16> {
        [
            self.request(gimbal, stale_flag, rtype, index, target::YAW),
            self.request(gimbal, stale_flag, rtype, index, target::PITCH),
        ]
    }

    pub fn value_received(&mut self, data: GimbalValueData) {
        let index = data.addr.index as usize;
        let target = data.addr.target as usize;
        if index < fygimbal::protocol::NUM_VALUES && target < fygimbal::protocol::NUM_AXES {
            self.values[index][target].last_update = Some((Instant::now(), data));
        }
    }
}
