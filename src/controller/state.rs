use bus::*;
use std::time::{Instant, Duration};
use config::{Config, ControllerMode};
use controller::manual::ManualControls;
use controller::winch::WinchController;
use led::{LightAnimator, LightEnvironment};

pub struct ControllerState {
    pub manual: ManualControls,
    lights: LightAnimator,
    winches: Vec<WinchController>,
    last_per_tick_update: Option<Instant>,
}

impl ControllerState {
    pub fn new(config: &Config, lights: LightAnimator) -> ControllerState {
        ControllerState {
            lights,
            manual: ManualControls::new(),
            winches: config.winches.iter().enumerate().map(|(id, _config)| {
                WinchController::new(id)
            }).collect(),
            last_per_tick_update: None,
        }
    }

    pub fn after_each_message(&mut self, timestamp: Instant, config: &Config) {
        let tick_duration = Duration::new(0, 1000000000 / TICK_HZ);
        let tick_has_elapsed = match self.last_per_tick_update {
            None => true,
            Some(inst) => (timestamp >= inst + tick_duration),
        };
        if tick_has_elapsed {
            self.every_tick(config);
            self.last_per_tick_update = Some(timestamp);
        }
    }

    fn lighting_tick(&mut self, config: &Config) {
        let env = self.light_environment(config);
        self.lights.update(env);
    }

    fn every_tick(&mut self, config: &Config) {
        self.manual.control_tick(config);
        self.lighting_tick(config);
    }

    pub fn flyer_sensor_update(&mut self, _sensors: FlyerSensors) {
    }

    pub fn winch_control_loop(&mut self, config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        self.winches[id].update(config, cal, &status);
        self.winches[id].velocity_tick(config, cal, match config.mode {

            ControllerMode::ManualWinch(manual_id) => {
                if manual_id == id {
                    let vec = self.manual.limited_velocity();
                    vec[1]
                } else {
                    0.0
                }
            },

            _ => 0.0
        });
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
