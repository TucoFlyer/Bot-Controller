use message::*;
use vecmath::*;
use std::time::{Instant, Duration};
use config::{Config, ControllerMode, GimbalTrackingGain};
use std::collections::HashMap;
use controller::velocity::RateLimitedVelocity;

pub struct ManualControls {
    axes: HashMap<ManualControlAxis, f32>,
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

    fn lookup_axis(&mut self, axis: ManualControlAxis) -> f32 {
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

    pub fn tracking_update(&mut self, config: &Config, rect: Vector4<f32>, time_step: f32) -> Vector4<f32> {
        let vec = self.camera_vector();
        let vec = if ManualControls::camera_vector_in_deadzone(vec, config) { [0.0, 0.0] } else { vec };
        let velocity = vec2_mul(vec, vec2_scale([1.0, -1.0], config.vision.manual_control_speed));

        let restoring_force_axis = |gains: &Vec<GimbalTrackingGain>, lower_dist: f32, upper_dist: f32| {
            let mut f = 0.0;
            for index in 0 .. gains.len() {
                let width = gains[index].width * config.vision.manual_control_restoring_force_width;
                f += (width - lower_dist).max(0.0) - (width - upper_dist).max(0.0);
            }
            f.max(-1.0).min(1.0) * config.vision.manual_control_restoring_force
        };

        let border = config.vision.border_rect;
        let left_dist = rect_left(rect) - rect_left(border);
        let top_dist = rect_top(rect) - rect_top(border);
        let right_dist = rect_right(border) - rect_right(rect);
        let bottom_dist = rect_bottom(border) - rect_bottom(rect);

        let restoring_force = [
            restoring_force_axis(&config.gimbal.yaw_gains, left_dist, right_dist),
            restoring_force_axis(&config.gimbal.pitch_gains, top_dist, bottom_dist),
        ];

        let center = vec2_add(rect_center(rect), vec2_scale(vec2_add(velocity, restoring_force), time_step));

        let side_len = config.vision.tracking_default_area.sqrt();
        let default_rect = rect_centered_on_origin(side_len, side_len);
        rect_constrain(rect_translate(default_rect, center), config.vision.border_rect)
    }

    pub fn camera_control_active(&self) -> bool {
        match self.camera_control_active_until_timestamp {
            None => false,
            Some(timestamp) => Instant::now() < timestamp,
        }
    }

    fn lookup_relative_vec(&mut self) -> Vector3<f32> {
        [
            self.lookup_axis(ManualControlAxis::RelativeX),
            self.lookup_axis(ManualControlAxis::RelativeY),
            self.lookup_axis(ManualControlAxis::RelativeZ),
        ]
    }

    fn velocity_target(&mut self, config: &Config) -> Vector3<f32> {
        let velocity_scale = config.params.manual_control_velocity_m_per_sec;
        vec3_scale(self.lookup_relative_vec(), velocity_scale)
    }

    pub fn limited_velocity(&self) -> Vector3<f32> {
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

    pub fn control_value(&mut self, axis: ManualControlAxis, value: f32) {
        self.axes.insert(axis, value);
    }

    pub fn control_reset(&mut self) {
        self.axes.clear();
    }
}
