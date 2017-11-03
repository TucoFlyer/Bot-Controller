use message::*;
use vecmath::*;
use std::time::{Duration, Instant};
use config::{Config, ControllerMode};
use controller::manual::ManualControls;
use controller::winch::{WinchController, MechStatus};
use led::{LightAnimator, LightEnvironment};
use overlay::DrawingContext;

pub struct ControllerState {
    pub manual: ManualControls,
    pub tracked: CameraTrackedRegion,
    lights: LightAnimator,
    winches: Vec<WinchController>,
    flyer_sensors: Option<FlyerSensors>,
    detected: (Instant, CameraDetectedObjects),
    pending_snap: bool,
    last_mode: ControllerMode,
}

impl ControllerState {
    pub fn new(initial_config: &Config, lights: LightAnimator) -> ControllerState {
        ControllerState {
            lights,
            manual: ManualControls::new(),
            winches: initial_config.winches.iter().enumerate().map(|(id, _config)| {
                WinchController::new(id)
            }).collect(),
            flyer_sensors: None,
            detected: (Instant::now(), CameraDetectedObjects::new()),
            pending_snap: false,
            tracked: CameraTrackedRegion::new(),
            last_mode: initial_config.mode.clone(),
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

    fn lighting_tick(&mut self, config: &Config) {
        let env = self.light_environment(config);
        self.lights.update(env);
    }

    fn default_tracking_rect(config: &Config) -> Vector4<f32> {
        let side_len = config.vision.tracking_default_area.sqrt();
        [side_len * -0.5, side_len * -0.5, side_len, side_len]
    }

    pub fn tracking_update(&mut self, config: &Config, time_step: f32) -> Option<Vector4<f32>> {
        if self.manual.camera_control_active() {
            // Manual tracking control temporarily overrides other sources
            let camera_vec = self.manual.camera_vector();
            let deadzone = ManualControls::camera_vector_in_deadzone(camera_vec, config);
            let camera_vec = if deadzone { [0.0, 0.0] } else { camera_vec };
            let velocity = vec2_mul(camera_vec, vec2_scale([1.0, -1.0], config.vision.manual_control_speed));
            let restoring_force = vec2_scale([-1.0, -1.0], config.vision.manual_control_restoring_force);
            let velocity = vec2_add(velocity, vec2_mul(rect_center(self.tracked.rect), restoring_force));
            let center = vec2_add(rect_center(self.tracked.rect), vec2_scale(velocity, time_step));
            self.tracked.rect = rect_translate(ControllerState::default_tracking_rect(config), center);
            self.tracked.rect = rect_constrain(self.tracked.rect, config.vision.border_rect);
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
        self.lighting_tick(config);
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

    pub fn draw_camera_overlay(&self, config: &Config, draw: &mut DrawingContext) {
        if config.mode == ControllerMode::Halted {
            draw.current.outline_color = config.overlay.halt_color;
            draw.current.outline_thickness = config.overlay.border_thickness;
            draw.outline_rect(rect_offset(config.vision.border_rect, -config.overlay.border_thickness));
        }

        draw.current.color = config.overlay.debug_color;
        draw.current.text_height = config.overlay.debug_text_height;
        draw.current.background_color = config.overlay.debug_background_color;
        draw.current.outline_thickness = 0.0;
        let debug = format!("{:?}", config.mode);
        draw.text(rect_topleft(config.vision.border_rect), [0.0, 0.0], &debug).unwrap();

        draw.current.outline_color = config.overlay.detector_default_outline_color;
        for obj in &self.detected.1.objects {
            if obj.prob >= config.overlay.detector_outline_min_prob {
                draw.current.outline_thickness = obj.prob * config.overlay.detector_outline_max_thickness;
                draw.outline_rect(obj.rect);
                let outer_rect = rect_offset(obj.rect, draw.current.outline_thickness);

                if obj.prob >= config.overlay.detector_label_min_prob {
                    draw.current.text_height = config.overlay.label_text_size;
                    draw.current.color = config.overlay.label_color;
                    draw.current.background_color = config.overlay.label_background_color;
                    draw.current.outline_thickness = 0.0;

                    let label = if config.overlay.detector_label_prob_values {
                        format!("{} p={:.3}", obj.label, obj.prob)
                    } else {
                        obj.label.clone()
                    };

                    draw.text(rect_topleft(outer_rect), [0.0, 1.0], &label).unwrap();
                }
            }
        }

        if !self.tracked.is_empty() {
            draw.current.outline_thickness = config.overlay.tracked_region_outline_thickness;

            if self.manual.camera_control_active() {
                draw.current.outline_color = config.overlay.tracked_region_manual_color;
                draw.outline_rect(self.tracked.rect);
    
            } else {
                draw.current.outline_color = config.overlay.tracked_region_default_color;
                draw.outline_rect(self.tracked.rect);

                let outer_rect = rect_offset(self.tracked.rect, config.overlay.tracked_region_outline_thickness);

                let tr_label = format!("psr={:.2} age={} area={:.3}",
                    self.tracked.psr, self.tracked.age, rect_area(self.tracked.rect));

                draw.current.text_height = config.overlay.label_text_size;
                draw.current.color = config.overlay.label_color;
                draw.current.background_color = config.overlay.label_background_color;
                draw.current.outline_thickness = 0.0;
                draw.text(rect_bottomleft(outer_rect), [0.0, 0.0], &tr_label).unwrap();
            }
        }
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
                    let v = self.manual.limited_velocity()[1];
                    match self.winches[id].mech_status {
                        MechStatus::Normal => v,
                        MechStatus::Stuck => 0.0,
                        MechStatus::ForceLimited(f) => if v * f < 0.0 { v } else { 0.0 },
                    }
                } else {
                    0.0
                }
            },

            _ => 0.0
        };

        self.winches[id].velocity_tick(config, cal, velocity);
        self.winches[id].make_command(config, cal, &status)
    }

    pub fn light_environment(&self, config: &Config) -> LightEnvironment {
        let winches = self.winches.iter().map( |winch| {
            winch.light_environment(&config)
        }).collect();

        LightEnvironment {
            winches,
            winch_wavelength: config.lighting.current.winch.wavelength_m,
            winch_wave_window_length: config.lighting.current.winch.wave_window_length_m,
            winch_wave_exponent: config.lighting.current.winch.wave_exponent,
            winch_command_color: config.lighting.current.winch.command_color,
            winch_motion_color: config.lighting.current.winch.motion_color,
            flash_exponent: config.lighting.current.flash_exponent,
            flash_rate_hz: config.lighting.current.flash_rate_hz,
            brightness: config.lighting.current.brightness,
        }
    }
}
