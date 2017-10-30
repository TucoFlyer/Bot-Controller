use message::*;
use vecmath::*;
use config::{Config, ControllerMode};
use controller::manual::ManualControls;
use controller::winch::{WinchController, MechStatus};
use led::{LightAnimator, LightEnvironment};
use overlay::DrawingContext;
use fygimbal;
use fygimbal::{GimbalPort, GimbalValueData};

pub struct ControllerState {
    pub manual: ManualControls,
    lights: LightAnimator,
    winches: Vec<WinchController>,
    flyer_sensors: Option<FlyerSensors>,
    detected: CameraDetectedObjects,
    tracked: CameraTrackedRegion,
    tracking_since_frame: u32,
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
            detected: CameraDetectedObjects::new(),
            tracked: CameraTrackedRegion::new(),
            tracking_since_frame: 0,
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

    fn gimbal_tick(&mut self, _config: &Config, gimbal: &GimbalPort) {
        // We always want to keep the yaw and pitch angles updated
        gimbal.request_continuous(fygimbal::protocol::values::ENCODER_ANGLES, fygimbal::protocol::target::YAW);
        gimbal.request_continuous(fygimbal::protocol::values::ENCODER_ANGLES, fygimbal::protocol::target::PITCH);
    }

    pub fn every_tick(&mut self, config: &Config, gimbal: &GimbalPort) {
        self.manual.control_tick(config);
        self.lighting_tick(config);
        self.gimbal_tick(config, gimbal);
    }

    fn find_best_snap_object(&self, config: &Config) -> Option<CameraDetectedObject> {
        let mut result = None;
        for obj in &self.detected.objects {
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

    pub fn camera_object_detection_loop(&mut self, config: &Config, det: CameraDetectedObjects) -> Option<Vector4<f32>> {
        self.detected = det;

        let best_snap = self.find_best_snap_object(config);
        if let Some(obj) = best_snap {
            self.tracked.rect = obj.rect;
            self.tracked.frame = self.detected.frame;
            self.tracking_since_frame = self.detected.frame;
            Some(obj.rect)
        } else {
            None
        }
    }

    pub fn camera_region_tracking_update(&mut self, config: &Config, tr: CameraTrackedRegion) -> Option<Vector4<f32>> {
        if (tr.frame.wrapping_sub(self.tracked.frame) as i32) <= 0 {
            // Already have a newer prediction or a prediction from the same frame (i.e. the object detector)
            // Just save the PSR
            self.tracked.psr = tr.psr;
        } else {
            self.tracked = tr;

            // If our tracking quality is very bad, reset to the center of the screen
            let area = rect_area(self.tracked.rect);
            let vis = &config.vision;
            if self.tracked.psr < vis.tracking_min_psr || area < vis.tracking_min_area {
                let side_len = vis.tracking_default_area.sqrt();
                return Some([side_len * -0.5, side_len * -0.5, side_len, side_len]);
            }
        }
        None
    }

    pub fn draw_camera_overlay(&self, config: &Config, draw: &mut DrawingContext) {

        draw.current.color = config.overlay.debug_color;
        draw.current.text_height = config.overlay.debug_text_height;
        let debug = format!("{:?}\n{:?}\n{:?}", config.mode, self.winches, self.flyer_sensors);
        draw.text([-1.0, -9.0/16.0], [0.0, 0.0], &debug).unwrap();

        draw.current.outline_color = config.overlay.detector_default_outline_color;
        for obj in &self.detected.objects {
            if obj.prob >= config.overlay.detector_outline_min_prob {
                draw.current.outline_thickness = obj.prob * config.overlay.detector_outline_max_thickness;
                draw.outline_rect(obj.rect);
            }

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

                draw.text_box(rect_topleft(obj.rect), [0.0, 0.0], &label).unwrap();
            }
        }

        if !self.tracked.is_empty()
            && config.overlay.tracked_region_outline_color[3] > 0.0
            && config.overlay.tracked_region_outline_thickness > 0.0 {

            draw.current.outline_color = config.overlay.tracked_region_outline_color;
            draw.current.outline_thickness = config.overlay.tracked_region_outline_thickness;
            draw.outline_rect(self.tracked.rect);

            let tr_label = format!("psr={:.2} age={} area={:.3}",
                self.tracked.psr, self.tracked.frame.wrapping_sub(self.tracking_since_frame),
                rect_area(self.tracked.rect));

            draw.current.text_height = config.overlay.label_text_size;
            draw.current.color = config.overlay.label_color;
            draw.current.background_color = config.overlay.label_background_color;
            draw.current.outline_thickness = 0.0;
            draw.text_box(rect_topright(self.tracked.rect), [1.0, 0.0], &tr_label).unwrap();
        }
    }

    pub fn gimbal_value_received(&mut self, _data: GimbalValueData, _gimbal: &GimbalPort) {
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
