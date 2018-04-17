use message::*;
use vecmath::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use config::{Config, ControllerMode};
use controller::manual::ManualControls;
use controller::winch::{WinchController, MechStatus};
use overlay::ParticleDrawing;
use led::WinchLighting;

pub struct ControllerState {
    pub manual: ManualControls,
    pub tracked: CameraTrackedRegion,
    pub detected: (Instant, CameraDetectedObjects),
    pub tracking_particles: ParticleDrawing,
    winches: Vec<WinchController>,
    flyer_sensors: Option<FlyerSensors>,
    camera_outputs: HashMap<CameraOutput, CameraOutputStatus>,
    pending_snap: bool,
    last_mode: ControllerMode,
}

impl ControllerState {
    pub fn new(initial_config: &Config) -> ControllerState {
        ControllerState {
            manual: ManualControls::new(),
            winches: initial_config.winches.iter().enumerate().map(|(id, _config)| {
                WinchController::new(id)
            }).collect(),
            flyer_sensors: None,
            detected: (Instant::now(), CameraDetectedObjects::new()),
            pending_snap: false,
            tracked: CameraTrackedRegion::new(),
            tracking_particles: ParticleDrawing::new(),
            last_mode: initial_config.mode.clone(),
            camera_outputs: HashMap::new(),
        }
    }

    pub fn config_changed(&mut self, config: &Config) {
        if config.mode != self.last_mode {
            self.mode_changed(&config.mode);
            self.last_mode = config.mode.clone();
        }
    }

    fn mode_changed(&mut self, _mode: &ControllerMode) {
        self.halt_motion();
    }

    fn halt_motion(&mut self) {
        self.manual.full_reset();
    }

    pub fn winch_lighting(&self, config: &Config) ->  Vec<WinchLighting> {
        self.winches.iter().map( |winch| {
            winch.light_environment(config)
        }).collect()
    }

    pub fn tracking_update(&mut self, config: &Config, time_step: f32, reset_tracking: bool) -> Option<Vector4<f32>> {
        if self.manual.camera_control_active() {
            // Manual tracking control temporarily overrides other sources
            self.tracked.rect = self.manual.tracking_update(config, self.tracked.rect, time_step);
            Some(self.tracked.rect)
        }
        else if reset_tracking {
            // Resets do not override manual controls, just everything else
            self.tracked = CameraTrackedRegion::new();
            Some(self.tracked.rect)
        }
        else if let Some(obj) = self.find_best_snap_object(config) {
            // Snap to a detected object
            self.pending_snap = false;
            self.tracked.rect = rect_constrain(obj.rect, config.vision.border_rect);
            self.tracked.frame = self.detected.1.frame;
            Some(self.tracked.rect)
        }
        else {
            None
        }
    }

    pub fn every_tick(&mut self, config: &Config) {
        self.manual.control_tick(config);
        self.tracking_particles.follow_rect(config, self.tracked.rect);
    }

    fn find_best_snap_object(&self, config: &Config) -> Option<CameraDetectedObject> {
        if !self.pending_snap {
            // No data from the CV subsystem yet or we've already processed the latest frame
            return None;
        }

        if self.detected.0 + Duration::from_millis(500) < Instant::now() {
            // Latest data from CV is too old to bother with
            return None;
        }

        if config.mode == ControllerMode::Halted {
            // No automatic CV activity during halt
            return None;
        }

        let mut result = None;
        for obj in &self.detected.1.objects {
            let area = rect_area(obj.rect);
            if area >= config.vision.tracking_min_area && area <= config.vision.tracking_max_area {
                for rule in &config.vision.snap_tracked_region_to {
                    if obj.prob >= rule.1 && obj.label == rule.0 {
                        result = match result {
                            None => Some(obj),
                            Some(prev) => if obj.prob > prev.prob { Some(obj) } else { Some(prev) }
                        };
                        break;
                    }
                }
            }
        }
        match result {
            None => None,
            Some(obj) => Some(obj.clone())
        }
    }

    pub fn camera_object_detection_update(&mut self, det: CameraDetectedObjects) {
        self.detected = (Instant::now(), det);
        self.pending_snap = true;
    }

    pub fn camera_region_tracking_update(&mut self, tr: CameraTrackedRegion) {
        if !self.manual.camera_control_active() {
            self.tracked = tr;
        }
    }

    pub fn camera_output_status_update(&mut self, outputs: HashMap<CameraOutput, CameraOutputStatus>) {
        self.camera_outputs = outputs;
    }

    pub fn flyer_sensor_update(&mut self, sensors: FlyerSensors) {
        self.flyer_sensors = Some(sensors);
    }

    pub fn winch_control_loop(&mut self, config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        self.winches[id].update(config, cal, &status);

        let velocity = match config.mode {

            ControllerMode::ManualWinch(manual_id) => {
                if manual_id == id {
                    self.manual_single_winch_controller(id)
                } else {
                    0.0
                }
            },

            ControllerMode::ManualFlyer => {
                self.manual_multi_winch_controller(config, id)
            }

            ControllerMode::Normal => {
                self.manual_multi_winch_controller(config, id)
            }

            ControllerMode::Halted => 0.0
        };

        self.winches[id].motion_tick_with_velocity(config, cal, velocity);
        self.winches[id].make_command(config, cal, &status)
    }

    fn force_limit_guard(&self, limit_direction: f32, velocity: f32) -> f32 {
        if velocity * limit_direction < 0.0 { velocity } else { 0.0 }
    }

    fn manual_single_winch_controller(&self, id: usize) -> f32 {
        let v = self.manual.limited_velocity()[1];
        match self.winches[id].mech_status {
            MechStatus::Stuck => 0.0,
            MechStatus::ForceLimited(f) => self.force_limit_guard(f, v),
            MechStatus::Normal => v,
        }
    }

    fn manual_multi_winch_controller(&self, config: &Config, id: usize) -> f32 {
        let v = self.manual.limited_velocity();
        let v = [v[0], -v[1], v[2]];
        self.multi_winch_controller(config, id, v)
    }

    fn multi_winch_controller(&self, config: &Config, id: usize, velocity: Vector3<f32>) -> f32 {
        let projected_velocity = vec3_dot(velocity, self.winch_rope_direction_vector(config, id));
        let return_v = config.params.force_return_velocity_max_m_per_sec;
        match self.winches[id].mech_status {
            MechStatus::Stuck => 0.0,
            MechStatus::ForceLimited(f) => self.force_limit_guard(f, projected_velocity) - f * return_v,
            MechStatus::Normal => projected_velocity,
        }
    }

    fn winch_rope_direction_vector(&self, config: &Config, id: usize) -> Vector3<f32> {
        // fix me
        vec3_normalized(config.winches[id].loc)
    }

    pub fn multi_winch_watchdog_should_halt(&self, config: &Config) -> bool {
        for winch in &self.winches {
            if !winch.is_status_recent(config) {
                // This winch isn't okay, halt unless we're in manual mode
                return match config.mode {
                    ControllerMode::Halted => false,
                    ControllerMode::ManualWinch(_) => false,
                    _ => true,
                };
            }
        }
        // All winches okay
        return false;
    }

    pub fn camera_output_is_active(&self, output: &CameraOutput) -> bool {
        if let Some(status) = self.camera_outputs.get(output) {
            status.active
        } else {
            false
        }
    }
}
