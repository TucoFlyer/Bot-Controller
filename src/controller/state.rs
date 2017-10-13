use message::*;
use config::{Config, ControllerMode};
use controller::manual::ManualControls;
use controller::winch::{WinchController, MechStatus};
use led::{LightAnimator, LightEnvironment};
use overlay::DrawingContext;

pub struct ControllerState {
    pub manual: ManualControls,
    lights: LightAnimator,
    winches: Vec<WinchController>,
    flyer_sensors: Option<FlyerSensors>,
    last_mode: ControllerMode,
}

impl ControllerState {
    pub fn new(config: &Config, lights: LightAnimator) -> ControllerState {
        ControllerState {
            lights,
            manual: ManualControls::new(),
            winches: config.winches.iter().enumerate().map(|(id, _config)| {
                WinchController::new(id)
            }).collect(),
            flyer_sensors: None,
            last_mode: config.mode.clone(),
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

    pub fn every_tick(&mut self, config: &Config) {
        self.manual.control_tick(config);
        self.lighting_tick(config);
    }

    pub fn draw_camera_overlay(&self, _config: &Config, draw: &mut DrawingContext) {
        let s = format!("Some text, yeah!!\n\nflyer_sensors:\n{:?}", self.flyer_sensors);
        draw.text_box([0.1, 0.1], &s).unwrap();
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
