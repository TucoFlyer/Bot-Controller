use message::*;
use vecmath::*;
use num::clamp;
use std::time::{Duration, Instant};
use config::{Config, ControllerMode};
use controller::manual::ManualControls;
use controller::winch::{WinchController, MechStatus};
use controller::gimbal::GimbalController;
use led::{LightAnimator, LightEnvironment};
use overlay::DrawingContext;
use fygimbal::{GimbalPort, GimbalValueData};

pub struct ControllerState {
    pub manual: ManualControls,
    lights: LightAnimator,
    winches: Vec<WinchController>,
    flyer_sensors: Option<FlyerSensors>,
    detected: (Instant, CameraDetectedObjects),
    pending_snap: bool,
    tracked: CameraTrackedRegion,
    last_mode: ControllerMode,
    gimbal: GimbalController,
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
            gimbal: GimbalController::new(),
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

    fn reset_tracking_rect(&mut self, config: &Config) {
        let side_len = config.vision.tracking_default_area.sqrt();
        self.tracked = CameraTrackedRegion::new();
        self.tracked.rect = [side_len * -0.5, side_len * -0.5, side_len, side_len];
    }

    pub fn tracking_update(&mut self, config: &Config, time_step: f32) -> Option<Vector4<f32>> {
        let vis = &config.vision;
        let area = rect_area(self.tracked.rect);
        let min_speed = vis.min_manual_control_speed;
        let max_speed = vis.max_manual_control_speed;
        let manual_control = self.manual.camera_vector();
        let manual_control = [manual_control[0] as f32 * max_speed, manual_control[1] as f32 * max_speed];
        let manual_control_active = manual_control[0].abs() > min_speed || manual_control[1].abs() > min_speed;
        let tracking_is_bad = (self.tracked.age > 0 && self.tracked.psr < vis.tracking_min_psr)
                || area < vis.tracking_min_area || area > vis.tracking_max_area;

        if manual_control_active {
            // Nudge the tracking rect
            self.tracked.rect[0] += manual_control[0] * time_step;
            self.tracked.rect[1] -= manual_control[1] * time_step;
            Some(self.tracked.rect)
        }
        else if let Some(obj) = self.find_best_snap_object(config) {
            // Snap to a detected object
            self.pending_snap = false;
            self.tracked.rect = obj.rect;
            self.tracked.frame = self.detected.1.frame;
            Some(self.tracked.rect)
        } 
        else if tracking_is_bad {
            // Reset to the default tracking rectangle
            self.reset_tracking_rect(config);
            Some(self.tracked.rect)
        }
        else {
            // Let the CV plugin keep tracking without being reset.
            // It only tracks on frames that aren't processing a reset.
            None
        }
    }

    pub fn every_tick(&mut self, config: &Config, gimbal: &GimbalPort) {
        self.manual.control_tick(config);
        self.lighting_tick(config);
        self.gimbal.tick(config, gimbal, &self.tracked);
    }

    fn find_best_snap_object(&self, config: &Config) -> Option<CameraDetectedObject> {
        let mut result = None;
        if self.pending_snap {
            if self.detected.0 + Duration::from_millis(500) > Instant::now() {
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
        if (tr.frame.wrapping_sub(self.tracked.frame) as i32) <= 0 {
            // Already have a newer prediction or a prediction from the same frame (i.e. the object detector)
            // Just save the PSR
            self.tracked.psr = tr.psr;
            self.tracked.age = 0;
        } else {
            self.tracked = tr;
        }
    }

    pub fn draw_camera_overlay(&self, config: &Config, draw: &mut DrawingContext) {
        if config.mode == ControllerMode::Halted {
            draw.current.outline_color = config.overlay.halt_color;
            draw.current.outline_thickness = config.overlay.border_thickness;
            draw.outline_rect(rect_offset(config.overlay.border_rect, -config.overlay.border_thickness));
        }

        draw.current.color = config.overlay.debug_color;
        draw.current.text_height = config.overlay.debug_text_height;
        let debug = format!("{:?}\n{}", config.mode, self.gimbal.debug_str);
        draw.text([-1.0, -9.0/16.0], [0.0, 0.0], &debug).unwrap();

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

                    draw.text_box(rect_topleft(outer_rect), [0.0, 1.0], &label).unwrap();
                }
            }
        }

        if !self.tracked.is_empty()
            && config.overlay.tracked_region_outline_color[3] > 0.0
            && config.overlay.tracked_region_outline_thickness > 0.0 {

            draw.current.outline_color = config.overlay.tracked_region_outline_color;
            draw.current.outline_thickness = config.overlay.tracked_region_outline_thickness;
            draw.outline_rect(self.tracked.rect);
            let outer_rect = rect_offset(self.tracked.rect, config.overlay.tracked_region_outline_thickness);

            let tr_label = format!("psr={:.2} age={} area={:.3}",
                self.tracked.psr, self.tracked.age, rect_area(self.tracked.rect));

            draw.current.text_height = config.overlay.label_text_size;
            draw.current.color = config.overlay.label_color;
            draw.current.background_color = config.overlay.label_background_color;
            draw.current.outline_thickness = 0.0;
            draw.text_box(rect_bottomleft(outer_rect), [0.0, 0.0], &tr_label).unwrap();
        }

        if config.overlay.gimbal_tracking_rect_color[3] > 0.0 {
            for &(rect, gain_vec) in &config.gimbal.tracking_rects {
                let overlap = rect_intersect(self.tracked.rect, rect);
                let area = rect_area(overlap);
                draw.current.color = config.overlay.gimbal_tracking_rect_color;
                draw.current.color[3] *= vec2_len(gain_vec);
                draw.current.color[3] *= clamp(area * config.overlay.gimbal_tracking_rect_sensitivity, 0.0, 1.0);
                draw.solid_rect(rect);
            }
        }
    }

    pub fn gimbal_value_received(&mut self, data: GimbalValueData) {
        self.gimbal.value_received(data);
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
