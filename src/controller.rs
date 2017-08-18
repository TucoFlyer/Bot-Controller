//! Controller thread, responding to commands and status updates, generating motor control packets


use bus::{Bus, Message, Command, FlyerSensors, WinchStatus, WinchCommand, ManualControlAxis};
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use multiqueue;
use config::BotConfig;
use botcomm::BotSender;


struct Controller {
    bus: Bus,
    bot_sender: BotSender,
    state: ControllerState,
}

impl Controller {

    fn new(bus: Bus, bot_sender: BotSender, config: BotConfig) -> Controller {
        Controller {
            bus,
            bot_sender,
            state: ControllerState {
                debug_control_axis: 0.0,                
            }
        }
    }

    fn poll(self: &mut Controller) {
        if let Ok(timestampedMessage) = self.bus.receiver.recv() {
            match timestampedMessage.message {

                Message::WinchStatus(id, status) => {
                    // Each winch status update results in a command packet to close the loop
                    self.bot_sender.winch_command(id, self.state.winch_control_loop(id, status));
                },

                Message::FlyerSensors(sensors) => {
                    // Flyer sensor input stores state for collision avoidance
                    self.state.flyer_sensor_update(sensors);
                },

                Message::Command( Command::ManualControlValue( ManualControlAxis::RelativeZ, v )) => {
                    self.state.debug_control_axis = v;
                },

                Message::Command( Command::ManualControlReset ) => {
                    self.state.debug_control_axis = 0.0;
                },

                _ => (),

            }
        }
    }
}

struct ControllerState {
    debug_control_axis: f32,
}

impl ControllerState {

    fn flyer_sensor_update(self: &mut ControllerState , sensors: FlyerSensors) {
        // collision avoidance and navigation feedback here, probably no direct packet sends as a result?
    }

    fn winch_control_loop(self: &mut ControllerState, id: usize, status: WinchStatus) -> WinchCommand {

        WinchCommand {
            velocity_target: (self.debug_control_axis * 4096.0) as i32,
            accel_max: 100,
            force_min: -50000,
            force_max: 700000,
        }
    }

}


pub fn start(bus: Bus, botsender: BotSender, config: BotConfig) {
    thread::spawn(move || {
        let mut controller = Controller::new(bus, botsender, config);
        loop {
            controller.poll();
        }
    });
}
