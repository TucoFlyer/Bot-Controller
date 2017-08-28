use bus::*;
use vecmath::*;
use config::Config;
use std::collections::HashMap;
use controller::velocity::RateLimitedVelocity;

pub struct ManualControls {
    axes: HashMap<ManualControlAxis, f64>,
    velocity: RateLimitedVelocity,
}

impl ManualControls {
    pub fn new() -> ManualControls {
        ManualControls {
            axes: HashMap::new(),
            velocity: RateLimitedVelocity::new(),
        }
    }

    fn lookup_axis(self: &mut ManualControls, axis: ManualControlAxis) -> f64 {
        self.axes.entry(axis).or_insert(0.0).min(1.0).max(-1.0)
    }

    fn lookup_relative_vec(self: &mut ManualControls) -> Vector3<f64> {
        [
            self.lookup_axis(ManualControlAxis::RelativeX),
            self.lookup_axis(ManualControlAxis::RelativeY),
            self.lookup_axis(ManualControlAxis::RelativeZ),
        ]
    }

    fn velocity_target(self: &mut ManualControls, config: &Config) -> Vector3<f64> {
        let velocity_scale = config.params.manual_control_velocity_m_per_sec;
        vec3_scale(self.lookup_relative_vec(), velocity_scale)
    }

    pub fn limited_velocity(self: &ManualControls) -> Vector3<f64> {
        self.velocity.get()
    }

    pub fn control_tick(self: &mut ManualControls, config: &Config) {
        let v = self.velocity_target(config);
        self.velocity.tick(config, v);
    }

    pub fn control_value(self: &mut ManualControls, axis: ManualControlAxis, value: f64) {
        self.axes.insert(axis, value);
    }

    pub fn control_reset(self: &mut ManualControls) {
        self.axes.clear();
    }
}
