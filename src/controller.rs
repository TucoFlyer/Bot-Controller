//! Controller thread, responding to commands and status updates, generating motor control packets


use bus::{Bus, Message, Command, FlyerSensors, WinchStatus, WinchCommand, ManualControlAxis};
use std::thread;
use config::BotConfig;
use botcomm::BotSender;
use std::collections::HashMap;

struct Controller {
    bus: Bus,
    bot_sender: BotSender,
    config: BotConfig,
    state: ControllerState,
}

struct ControllerState {
   manual_controls: HashMap<ManualControlAxis, f64>,
}


impl Controller {

    fn new(bus: Bus, bot_sender: BotSender, config: BotConfig) -> Controller {
        Controller {
            bus,
            bot_sender,
            config,
            state: ControllerState {
                manual_controls: HashMap::new(),
            }
        }
    }

    fn poll(self: &mut Controller) {
        if let Ok(ts_msg) = self.bus.receiver.recv() {
            match ts_msg.message {

                Message::WinchStatus(id, status) => {
                    drop(self.bot_sender.winch_command(id, self.state.winch_control_loop(id, status, &self.config)));
                },

                Message::FlyerSensors(sensors) => {
                    self.state.flyer_sensor_update(sensors);
                },

                Message::Command(Command::ManualControlValue( axis, value )) => {
                    self.state.manual_controls.insert(axis, value);
                },

                Message::Command(Command::ManualControlReset ) => {
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

    fn winch_control_loop(self: &mut ControllerState, id: usize, status: WinchStatus, config: &BotConfig) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        WinchCommand {
            velocity_target: cal.dist_from_m(self.winch_velocity_target_m(id, status, config)) as f32,
            accel_rate: cal.dist_from_m(config.params.accel_rate_m_per_sec2) as f32,
            force_min: cal.force_from_kg(config.params.force_min_kg) as f32,
            force_max: cal.force_from_kg(config.params.force_max_kg) as f32,
            force_filter_param: config.params.force_filter_param,
            pwm_gain_p: config.params.pwm_gain_p,
            pwm_gain_i: config.params.pwm_gain_i,
            pwm_gain_d: config.params.pwm_gain_d,
        }
    }

    fn winch_velocity_target_m(self: &mut ControllerState, id: usize, status: WinchStatus, config: &BotConfig) -> f64 {
        let manual_y = *self.manual_controls.entry(ManualControlAxis::RelativeY).or_insert(0.0);
        config.params.manual_control_velocity_m_per_sec * manual_y
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
