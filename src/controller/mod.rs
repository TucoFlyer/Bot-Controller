//! Controller thread, responding to commands and status updates, generating motor control packets

mod manual;
mod velocity;
mod winch;
mod state;

use bus::*;
use std::thread;
use config::ConfigFile;
use botcomm::BotComm;
use self::state::ControllerState;
use led::LightAnimator;

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

impl Controller {
    fn new(bus: Bus, comm: BotComm, cf: ConfigFile) -> Controller {
        let lights = LightAnimator::start(&cf.config.lighting.animation, &comm);
        let state = ControllerState::new(&cf.config, lights);
        Controller { bus, comm, cf, state }
    }

    fn config_changed(self: &mut Controller) {
        *self.bus.config.lock().unwrap() = self.cf.config.clone();
        drop(self.bus.sender.try_send(Message::ConfigIsCurrent(self.cf.config.clone()).timestamp()));
        self.cf.save_async();
    }

    fn poll(self: &mut Controller) {
        if let Ok(ts_msg) = self.bus.receiver.recv() {
            let timestamp = ts_msg.timestamp;
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
                    self.state.manual.control_value(axis, value);
                },

                Message::Command(Command::ManualControlReset) => {
                    self.state.manual.control_reset();
                },

                _ => (),
            }
            self.state.after_each_message(timestamp, &self.cf.config);
        }
    }
}
