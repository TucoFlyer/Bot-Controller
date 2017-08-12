//! Controller thread, responding to commands and status updates, generating motor control packets


use bus::{Bus, Message, Command, WinchCommand, ManualControlAxis};
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use multiqueue;
use config::BotConfig;
use botcomm::BotSender;


struct Controller {
    bus: Bus,
    bot_sender: BotSender,
}


impl Controller {

    fn new(bus: Bus, bot_sender: BotSender, config: BotConfig) -> Controller {
        Controller { bus, bot_sender }
    }

    fn poll(self: &mut Controller) {
        match self.bus.receiver.recv() {

            Ok(Message::WinchStatus(id, status)) => {
                //println!("ctrl {:?} = {:?}", id, status);
            },

            Ok(Message::FlyerSensors(sensors)) => {
                //println!("ctrl {:?}", sensors);
            },

            Ok(Message::Command( Command::ManualControlValue( ManualControlAxis::RelativeZ, v ))) => {
                self.simple_control(v);
            }

            Ok(Message::Command( Command::ManualControlReset )) => {
                self.simple_control(0.0);
            }

            _ => (),
        }
    }

    fn simple_control(self: &mut Controller, v: f32) {

        let cmd = WinchCommand {
            velocity_target: (v * 4096.0) as i32,
            accel_max: 100,
            force_min: 10,
            force_max: 10000,
        };

        println!("{:?}", cmd);
        self.bot_sender.winch_command(0, cmd);
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
