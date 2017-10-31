use vecmath::*;
use message::CameraTrackedRegion;
use config::{ControllerMode, Config};
use fygimbal;
use fygimbal::protocol::{target, values};
use fygimbal::{GimbalPort, GimbalValueData};
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
    pub debug_str: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum RequestType {
    Continuous,
    Infrequent,
}

fn encoder_sub(a: i16, b: i16) -> i16 {
    const RANGE : i16 = 4096;
    let c = a.wrapping_sub(b) % RANGE;
    if c < -RANGE/2 {
        c + RANGE
    } else if c > RANGE/2 {
        c - RANGE
    } else {
        c
    }
}

fn vec2_encoder_sub(a: Vector2<i16>, b: Vector2<i16>) -> Vector2<i16> {
    [ encoder_sub(a[0], b[0]), encoder_sub(a[1], b[1]) ]
}

fn endstop_rate_limiter(config: &Config, angles: Vector2<i16>, rates: Vector2<f32>) -> Vector2<i16> {
    [
        single_endstop_limit(config, angles[0], rates[0], config.gimbal.yaw_limits),
        single_endstop_limit(config, angles[1], rates[1], config.gimbal.pitch_limits),
    ]
}

fn single_endstop_limit(config: &Config, angle: i16, rate: f32, limits: (i16, i16)) -> i16 {
    let rate = if angle < limits.0 {
        config.gimbal.limiter_gain * (limits.0 - angle) as f32
    } else if angle > limits.1 {
        config.gimbal.limiter_gain * (limits.1 - angle) as f32
    } else {
        rate
    };
    rate.min(config.gimbal.max_rate).max(-config.gimbal.max_rate).round() as i16
}

impl GimbalController {
    pub fn new() -> GimbalController {
        GimbalController {
            debug_str: "".to_owned(),
            values: (0 .. fygimbal::protocol::NUM_VALUES).map(|_| {
                (0 .. fygimbal::protocol::NUM_AXES).map(|_| {
                    GimbalValueState::new()
                 }).collect()
            }).collect()
        }
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

    pub fn tick(&mut self, config: &Config, gimbal: &GimbalPort, tracked: &CameraTrackedRegion) {
        let mut stale_flag = false;

        let center_cal = self.request_vec2(gimbal, &mut stale_flag, RequestType::Infrequent, values::CENTER_CALIBRATION);
        let raw_angles = self.request_vec2(gimbal, &mut stale_flag, RequestType::Continuous, values::ENCODER_ANGLES);
        let angles = vec2_encoder_sub(raw_angles, center_cal);

        let rates = if stale_flag || config.mode == ControllerMode::Halted {
            [ 0, 0 ]
        } else {
            endstop_rate_limiter(config, angles, self.control_rates_for_tracking(config, tracked))
        };

        self.debug_str = format!("angles: {:?}\nrates: {:?}", angles, rates);

        gimbal.write_control_rates(rates); 
    }

    fn control_rates_for_tracking(&self, config: &Config, tracked: &CameraTrackedRegion) -> Vector2<f32> {
        // to do
        let center = rect_center(tracked.rect);
        [
            config.vision.tracking_gains[0] * center[0],
            config.vision.tracking_gains[1] * center[1]
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
