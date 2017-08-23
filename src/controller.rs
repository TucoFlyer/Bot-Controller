//! Controller thread, responding to commands and status updates, generating motor control packets


use bus::{Bus, Message, Command, FlyerSensors, WinchStatus, WinchCommand, ManualControlAxis};
use std::thread;
use config::{Config, ConfigFile, ControllerMode};
use botcomm::BotComm;
use std::collections::HashMap;

pub fn start(bus: &Bus, comm: &BotComm, cf: ConfigFile) {
    let bus = bus.clone();
    let comm = comm.try_clone().unwrap();
    thread::spawn(move || {
        let mut controller = Controller::new(bus, comm, cf);
        loop {
            controller.poll();
        }
    });
}

struct Controller {
    bus: Bus,
    comm: BotComm,
    cf: ConfigFile,
    state: ControllerState,
}

struct ControllerState {
    manual_controls: HashMap<ManualControlAxis, f64>,
}

impl Controller {

    fn new(bus: Bus, comm: BotComm, cf: ConfigFile) -> Controller {
        let state = ControllerState {
            manual_controls: HashMap::new(),
        }; 
        Controller { bus, comm, cf, state }
    }

    fn config_changed(self: &mut Controller) {
        *self.bus.config.lock().unwrap() = self.cf.config.clone();
        drop(self.bus.sender.try_send(Message::ConfigIsCurrent(self.cf.config.clone()).timestamp()));
        self.cf.save_async();
    }

    fn poll(self: &mut Controller) {
        if let Ok(ts_msg) = self.bus.receiver.recv() {
            match ts_msg.message {

                Message::UpdateConfig(updates) => {
                    // Merge a freeform update into the configuration, and save it.
                    // Errors here go right to the console, since errors caused by a
                    // client should have been detected earlier and sent to that client.
                    match self.cf.config.merge(updates) {
                        Err(e) => println!("Error in UpdateConfig from message bus: {}", e),
                        Ok(config) => {
                            self.cf.config = config;
                            self.config_changed();
                        }
                    }
                }

                Message::Command(Command::SetMode(mode)) => {
                    // The controller mode is part of the config, so this could be changed via UpdateConfig as well, but this option is strongly typed
                    if self.cf.config.mode != mode {
                        self.cf.config.mode = mode;
                        self.config_changed();
                    }
                }

                Message::WinchStatus(id, status) => {
                    drop(self.comm.winch_command(id, self.state.winch_control_loop(&self.cf.config, id, status)));
                },

                Message::FlyerSensors(sensors) => {
                    self.state.flyer_sensor_update(sensors);
                },

                Message::Command(Command::ManualControlValue(axis, value)) => {
                    self.state.manual_controls.insert(axis, value);
                },

                Message::Command(Command::ManualControlReset) => {
                    self.state.manual_controls.clear();
                },

                _ => (),
            }
        }
    }
}

impl ControllerState {

    fn flyer_sensor_update(self: &mut ControllerState, sensors: FlyerSensors) {

    }

    fn winch_control_loop(self: &mut ControllerState,  config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        WinchCommand {
            velocity_target: cal.dist_from_m(self.winch_velocity_target_m(config, id, status)) as f32,
            accel_rate: cal.dist_from_m(config.params.accel_rate_m_per_sec2) as f32,
            force_min: cal.force_from_kg(config.params.force_min_kg) as f32,
            force_max: cal.force_from_kg(config.params.force_max_kg) as f32,
            force_filter_param: config.params.force_filter_param as f32,
            pwm_gain_p: cal.pwm_gain_from_m(config.params.pwm_gain_p) as f32,
            pwm_gain_i: cal.pwm_gain_from_m(config.params.pwm_gain_i) as f32,
            pwm_gain_d: cal.pwm_gain_from_m(config.params.pwm_gain_d) as f32,
        }
    }

    fn winch_velocity_target_m(self: &mut ControllerState, config: &Config, id: usize, status: WinchStatus) -> f64 {
        match config.mode {

            ControllerMode::Halted => 0.0,

            ControllerMode::ManualWinch(manual_id) =>
                if manual_id == id {
                    let manual_y = self.manual_controls.entry(ManualControlAxis::RelativeY).or_insert(0.0).min(1.0).max(-1.0);
                    config.params.manual_control_velocity_m_per_sec * manual_y
                } else {
                    0.0
                },

            _ => 0.0,
        }
    }
}
