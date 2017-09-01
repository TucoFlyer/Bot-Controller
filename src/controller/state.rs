use bus::*;
use vecmath::vec3_to_rgb;
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
        let manual = ManualControls::new();
        let winches = config.winches.iter().map( |_| WinchController::new() ).collect();
        let last_per_tick_update = None;
        ControllerState { manual, lights, winches, last_per_tick_update }
    }

    pub fn after_each_message(&mut self, timestamp: Instant, config: &Config) {
        let tick_duration = Duration::new(0, 1000000000 / TICK_HZ);
        let tick_has_elapsed = match self.last_per_tick_update {
            None => true,
            Some(inst) => (timestamp.duration_since(inst) >= tick_duration),
        };
        if tick_has_elapsed {
            self.every_tick(config);
            self.last_per_tick_update = Some(timestamp);
        }
    }

    fn every_tick(&mut self, config: &Config) {
        let light_env = self.light_environment(config);
        self.lights.update(light_env);
    }

    pub fn flyer_sensor_update(&mut self, _sensors: FlyerSensors) {
    }

    pub fn winch_control_loop(&mut self, config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        self.winches[id].update(config, &status);
        self.winches[id].velocity_tick(cal, match config.mode {

            ControllerMode::ManualWinch(manual_id) => {
                if manual_id == id {
                    self.manual.control_tick(config);
                    let vec = self.manual.limited_velocity();
                    vec[1]
                } else {
                    0.0
                }
            },

            ControllerMode::Halted => {
                self.manual = ManualControls::new();
                0.0
            }

            _ => 0.0
        });
        self.winches[id].make_command(config, cal, &status)
    }

    pub fn light_environment(&self, config: &Config) -> LightEnvironment {
        LightEnvironment {
            rgb_brightness_scale: vec3_to_rgb(config.params.led_rgb_brightness_scale),
            ctrl_mode: config.mode.clone(),
            winches: self.winches.iter().map(|winch| {
                winch.light_environment(&config)
            }).collect(),
        }
    }
}
