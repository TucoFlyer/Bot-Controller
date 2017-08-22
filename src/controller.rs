//! Controller thread, responding to commands and status updates, generating motor control packets


use bus::{Bus, Message, Command, FlyerSensors, WinchStatus, WinchCommand, ManualControlAxis};
use std::thread;
use config::Config;
use botcomm::BotComm;
use std::collections::HashMap;

pub fn start(bus: &Bus, comm: &BotComm) {
    let bus = bus.clone();
    let comm = comm.try_clone().unwrap();
    thread::spawn(move || {
        let mut controller = Controller::new(bus, comm);
        loop {
            controller.poll();
        }
    });
}

struct Controller {
    bus: Bus,
    comm: BotComm,
    config: Config,
    state: ControllerState,
}

struct ControllerState {
   manual_controls: HashMap<ManualControlAxis, f64>,
}

impl Controller {

    fn new(bus: Bus, comm: BotComm) -> Controller {
        let config = bus.config.lock().unwrap().clone();
        let state = ControllerState {
            manual_controls: HashMap::new(),
        }; 
        Controller { bus, comm, config, state }
    }

    fn poll(self: &mut Controller) {
        if let Ok(ts_msg) = self.bus.receiver.recv() {
            match ts_msg.message {

                Message::WinchStatus(id, status) => {
                    drop(self.comm.winch_command(id, self.state.winch_control_loop(&self.config, id, status)));
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

    fn winch_control_loop(self: &mut ControllerState,  config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        WinchCommand {
            velocity_target: cal.dist_from_m(self.winch_velocity_target_m(config, id, status)) as f32,
            accel_rate: cal.dist_from_m(config.params.accel_rate_m_per_sec2) as f32,
            force_min: cal.force_from_kg(config.params.force_min_kg) as f32,
            force_max: cal.force_from_kg(config.params.force_max_kg) as f32,
            force_filter_param: config.params.force_filter_param,
            pwm_gain_p: config.params.pwm_gain_p,
            pwm_gain_i: config.params.pwm_gain_i,
            pwm_gain_d: config.params.pwm_gain_d,
        }
    }

    fn winch_velocity_target_m(self: &mut ControllerState, config: &Config, id: usize, status: WinchStatus) -> f64 {
        let manual_y = *self.manual_controls.entry(ManualControlAxis::RelativeY).or_insert(0.0);
        config.params.manual_control_velocity_m_per_sec * manual_y
    }
}
