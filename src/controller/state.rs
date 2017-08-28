use bus::*;
use config::{Config, ControllerMode};
use controller::manual::ManualControls;
use controller::winch::WinchController;

pub struct ControllerState {
    pub manual: ManualControls,
    winches: Vec<WinchController>,
}

impl ControllerState {
    pub fn new(config: &Config) -> ControllerState {
        ControllerState {
            manual: ManualControls::new(),
            winches: config.winches.iter().map( |_| WinchController::new() ).collect(),
        }
    }

    pub fn flyer_sensor_update(self: &mut ControllerState, _sensors: FlyerSensors) {
    }

    pub fn winch_control_loop(self: &mut ControllerState, config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        self.winches[id].update(config, &status);

        let velocity = match config.mode {

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
        };

        self.winches[id].velocity_tick(cal, velocity);
        self.winches[id].make_command(config, cal, &status)
    }
}
