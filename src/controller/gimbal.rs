use vecmath::*;
use message::*;
use config::{ControllerMode, Config, GimbalTrackingGain};
use fygimbal;
use fygimbal::protocol::{target, values, motor_status};
use fygimbal::util::vec2_encoder_sub;
use fygimbal::GimbalPort;
use std::time::{Duration, Instant};

pub struct GimbalController {
    values: Vec<Vec<GimbalValueState>>,
    yaw_tracking_i: Vec<f32>,
    pitch_tracking_i: Vec<f32>,
    hold_angles: Vector2<i16>,
    hold_active: Vector2<bool>,
    hold_i: Vector2<f32>,
    current_osc_detector: Vector3<f32>,
    current_peak_detector: Vector3<f32>,
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
            hold_angles: [0; 2],
            hold_active: [false; 2],
            hold_i: [0.0; 2],
            current_osc_detector: [0.0; 3],
            current_peak_detector: [0.0; 3],
        }
    }

    pub fn tick(&mut self, config: &Config, gimbal: &GimbalPort, tracked: &CameraTrackedRegion) -> GimbalControlStatus {
        let mut stale_flag = false;
        let supply_voltage = self.request(gimbal, &mut stale_flag, RequestType::Infrequent, values::SUPPLY_VOLTAGE, target::IMU_ADJACENT) as f32 * 0.1;
        let center_cal = self.request_vec2(gimbal, &mut stale_flag, RequestType::Infrequent, values::CALIBRATION_ANGLE_0_CENTER);
        let raw_angles = self.request_vec2(gimbal, &mut stale_flag, RequestType::Continuous, values::ENCODER_ANGLE);
        let current = self.request_vec3(gimbal, &mut stale_flag, RequestType::Continuous, values::MOTOR_FILTERED_CURRENT);
        let motor_status = self.request_vec3(gimbal, &mut stale_flag, RequestType::Infrequent, values::MOTOR_STATUS_FLAGS);
        let angles = vec2_encoder_sub(raw_angles, center_cal);

        let mut status = GimbalControlStatus {
            angles,
            rates: [0; 2],
            tracking_p_rates: [0.0; 2],
            tracking_i_rates: [0.0; 2],
            hold_p_rates: [0.0; 2],
            hold_i_rates: [0.0; 2],
            yaw_gain_activations: config.gimbal.yaw_gains.iter().map(|_| 0.0).collect(),
            pitch_gain_activations: config.gimbal.pitch_gains.iter().map(|_| 0.0).collect(),
            hold_angles: self.hold_angles,
            hold_active: self.hold_active,
            motor_power: vec3_nonzero(vec3_bitand(motor_status, [motor_status::POWER_ON; 3])),
            current,
            current_osc_detector: self.current_osc_detector,
            current_peak_detector: self.current_peak_detector,
            supply_voltage
        };

        if !stale_flag {
            self.tracking_tick(config, &mut status, tracked);
            self.hold_tick(config, &mut status);
            self.gimbal_rate_tick(config, &mut status);
        }

        gimbal.write_control_rates(status.rates);
        status
    }

    fn tracking_tick(&mut self, config: &Config, status: &mut GimbalControlStatus, tracked: &CameraTrackedRegion) {
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
                    // Halt or change directions; reduce integral gain accumulator
                    i_state[index] -= i_state[index] * config.gimbal.tracking_i_decay_rate;
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
    }

    fn hold_tick(&mut self, config: &Config, status: &mut GimbalControlStatus) {
        let next_hold_active = if config.mode == ControllerMode::Halted {
            // Always hold position in halt mode
            [true, true]
        } else {
            // Look for transition in/out of proportional gain region.
            // Ignore integral gain here, as it needs to persist across the transition into hold mode.
            [ status.tracking_p_rates[0] == 0.0, status.tracking_p_rates[1] == 0.0 ]
        };

        // Capture angles at the beginning of a hold
        if next_hold_active[0] && !self.hold_active[0] { self.hold_angles[0] = status.angles[0]; }
        if next_hold_active[1] && !self.hold_active[1] { self.hold_angles[1] = status.angles[1]; }
        self.hold_active = next_hold_active;
        status.hold_angles = self.hold_angles;
        status.hold_active = self.hold_active;

        let hold_err = vec2_sub(status.hold_angles, status.angles);

        for axis in 0..2 {
            let err = hold_err[axis] as f32;
            if next_hold_active[axis] {
                self.hold_i[axis] += err;
                status.hold_p_rates[axis] = err * config.gimbal.hold_p_gain;
            } else {
                self.hold_i[axis] -= self.hold_i[axis] * config.gimbal.hold_i_decay_rate;
                status.hold_p_rates[axis] = 0.0;
            }
            status.hold_i_rates[axis] = self.hold_i[axis] * config.gimbal.hold_i_gain;
        }
    }

    fn gimbal_rate_tick(&mut self, config: &Config, status: &mut GimbalControlStatus) {
        // In halt, tracking is disabled (but limiter and hold mode work)
        let tracking_rates = if config.mode == ControllerMode::Halted {
            [0.0, 0.0]
        } else {
            vec2_add(status.tracking_i_rates, status.tracking_p_rates)
        };

        let hold_rates = vec2_add(status.hold_i_rates, status.hold_p_rates);

        let rates = vec2_add(tracking_rates, hold_rates);
        let rates = self.limiter(config, status.angles, rates);
        let rates = vec2_clamp_len(rates, config.gimbal.max_rate);
        status.rates = self.dither_rates(rates);
    }

    fn limiter(&self, config: &Config, angles: Vector2<i16>, rates: Vector2<f32>) -> Vector2<f32> {
        let axis = |rate: f32, angle: i16, limits: (i16, i16), slowdown_extent: f32| {
            let angle = angle as f32;
            let limits = (limits.0 as f32, limits.1 as f32);

            if angle < limits.0 {
                // Past lower limit; spring back
                rate.max(0.0) + (limits.0 - angle) * config.gimbal.limiter_gain
            } else if angle > limits.1 {
                // Past upper limit; spring back
                rate.min(0.0) + (limits.1 - angle) * config.gimbal.limiter_gain
            } else if angle < limits.0 + slowdown_extent {
                // In lower slowdown region
                let speed_limit = (angle - limits.0) / slowdown_extent * config.gimbal.max_rate;
                rate.max(-speed_limit)
            } else if angle > limits.1 - slowdown_extent {
                let speed_limit = -(angle - limits.1) / slowdown_extent * config.gimbal.max_rate;
                rate.min(speed_limit)
            } else {
                rate
            }
        };
        [
            axis(rates[0], angles[0], config.gimbal.yaw_limits, config.gimbal.limiter_slowdown_extent[0]),
            axis(rates[1], angles[1], config.gimbal.pitch_limits, config.gimbal.limiter_slowdown_extent[1])
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
                if slot.poll_for_request(3000) {
                    gimbal.request_once(index, target);
                }
                slot.value_with_max_age(stale_flag, 5000)
            }
        }
    }

    fn request_vec2(&mut self, gimbal: &GimbalPort, stale_flag: &mut bool, rtype: RequestType, index: u8) -> Vector2<i16> {
        [
            self.request(gimbal, stale_flag, rtype, index, target::YAW),
            self.request(gimbal, stale_flag, rtype, index, target::PITCH),
        ]
    }

    fn request_vec3(&mut self, gimbal: &GimbalPort, stale_flag: &mut bool, rtype: RequestType, index: u8) -> Vector3<i16> {
        [
            self.request(gimbal, stale_flag, rtype, index, target::YAW),
            self.request(gimbal, stale_flag, rtype, index, target::ROLL),
            self.request(gimbal, stale_flag, rtype, index, target::PITCH),
        ]
    }

    pub fn value_received(&mut self, config: &Config, data: GimbalValueData) {
        let index = data.addr.index as usize;
        let target = data.addr.target as usize;

        if index < fygimbal::protocol::NUM_VALUES && target < fygimbal::protocol::NUM_AXES {
            let value = &mut self.values[index][target];

            if index == values::MOTOR_FILTERED_CURRENT as usize {
                if let Some((_, ref prev_data)) = value.last_update {
                    let positive_diff = (data.value as i32 - prev_data.value as i32).abs() as f32;
                    let accum = &mut self.current_osc_detector[target];
                    *accum = *accum - *accum * config.gimbal.current_osc_detector_decay_rate + positive_diff;
                }

                let peak_ptr = &mut self.current_peak_detector[target];
                let positive_current = data.value.abs() as f32;
                let peak_decay = *peak_ptr - *peak_ptr * config.gimbal.current_peak_detector_decay_rate;
                *peak_ptr = if positive_current > peak_decay {
                    peak_decay + (positive_current - peak_decay) * config.gimbal.current_peak_detector_update_rate
                } else {
                    peak_decay
                }
            }

            value.last_update = Some((Instant::now(), data));
        }
    }

    pub fn set_motor_enable(&mut self, gimbal: &GimbalPort, en: bool) {
        gimbal.set_motor_enable(en);
        gimbal.request_once(values::MOTOR_STATUS_FLAGS, target::YAW);
        gimbal.request_once(values::MOTOR_STATUS_FLAGS, target::ROLL);
        gimbal.request_once(values::MOTOR_STATUS_FLAGS, target::PITCH);
    }
}

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
