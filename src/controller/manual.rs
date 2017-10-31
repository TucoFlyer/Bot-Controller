use message::*;
use vecmath::*;
use std::time::{Instant, Duration};
use config::{Config, ControllerMode};
use std::collections::HashMap;
use controller::velocity::RateLimitedVelocity;

pub struct ManualControls {
    axes: HashMap<ManualControlAxis, f64>,
    velocity: RateLimitedVelocity,
    camera_control_active_until_timestamp: Option<Instant>,
}

impl ManualControls {
    pub fn new() -> ManualControls {
        ManualControls {
            axes: HashMap::new(),
            velocity: RateLimitedVelocity::new(),
            camera_control_active_until_timestamp: None,
        }
    }

    pub fn full_reset(&mut self) {
        *self = ManualControls::new();
    }

    fn lookup_axis(&mut self, axis: ManualControlAxis) -> f64 {
        self.axes.entry(axis).or_insert(0.0).min(1.0).max(-1.0)
    }

    pub fn camera_vector(&mut self) -> Vector2<f32> {
        vec2_cast([
            self.lookup_axis(ManualControlAxis::CameraYaw),
            self.lookup_axis(ManualControlAxis::CameraPitch),
        ])
    }

    pub fn camera_vector_in_deadzone(cam_vec: Vector2<f32>, config: &Config) -> bool {
        let z = config.vision.manual_control_deadzone;
        vec2_square_len(cam_vec) <= z*z
    }

    pub fn camera_control_active(&self) -> bool {
        match self.camera_control_active_until_timestamp {
            None => false,
            Some(timestamp) => Instant::now() < timestamp,
        }
    }

    fn lookup_relative_vec(&mut self) -> Vector3<f64> {
        [
            self.lookup_axis(ManualControlAxis::RelativeX),
            self.lookup_axis(ManualControlAxis::RelativeY),
            self.lookup_axis(ManualControlAxis::RelativeZ),
        ]
    }

    fn velocity_target(&mut self, config: &Config) -> Vector3<f64> {
        let velocity_scale = config.params.manual_control_velocity_m_per_sec;
        vec3_scale(self.lookup_relative_vec(), velocity_scale)
    }

    pub fn limited_velocity(&mut self) -> Vector3<f64> {
        self.velocity.get()
    }

    pub fn control_tick(&mut self, config: &Config) {
        match config.mode {
            ControllerMode::Halted => self.full_reset(),
            _ => {
                let v = self.velocity_target(config);
                self.velocity.tick(config, v);
                self.camera_control_tick(config);
            }
        };
    }

    pub fn camera_control_tick(&mut self, config: &Config) {
        if !ManualControls::camera_vector_in_deadzone(self.camera_vector(), config) {
            let timeout = Duration::from_millis((1000.0 * config.vision.manual_control_timeout_sec) as u64);
            self.camera_control_active_until_timestamp = Some(Instant::now() + timeout);
        }
    }

    pub fn control_value(&mut self, axis: ManualControlAxis, value: f64) {
        self.axes.insert(axis, value);
    }

    pub fn control_reset(&mut self) {
        self.axes.clear();
    }
}
