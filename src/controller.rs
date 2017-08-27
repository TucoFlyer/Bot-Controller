//! Controller thread, responding to commands and status updates, generating motor control packets

use bus::*;
use std::thread;
use config::{Config, ConfigFile, ControllerMode, WinchCalibration};
use botcomm::{BotComm, TICK_HZ};
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

impl Controller {
    fn new(bus: Bus, comm: BotComm, cf: ConfigFile) -> Controller {
        let state = ControllerState::new(&cf.config);
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

struct ControllerState {
    manual_controls: HashMap<ManualControlAxis, f64>,
    winches: Vec<WinchController>,
}

impl ControllerState {
    fn new(config: &Config) -> ControllerState {
        ControllerState {
            manual_controls: HashMap::new(),
            winches: config.winches.iter().map( |_| WinchController::new() ).collect(),
        }
    }

    fn flyer_sensor_update(self: &mut ControllerState, sensors: FlyerSensors) {

    }

    fn winch_control_loop(self: &mut ControllerState, config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        self.winches[id].update(config, id, &status);
        let vtarget = self.winch_velocity_target_m(config, id);
        self.winches[id].velocity_target_tick(cal, vtarget);
        self.winches[id].make_command(config, cal, &status)
    }

    fn winch_velocity_target_m(self: &mut ControllerState, config: &Config, id: usize) -> f64 {
        match config.mode {

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

struct WinchController {
    last_tick_counter: Option<u32>,
    quantized_position_target: i32,
    fract_position_target: f64,
}

impl WinchController {
    fn new() -> WinchController {
        WinchController {
            last_tick_counter: None,
            quantized_position_target: 0,
            fract_position_target: 0.0
        }
    }

    fn velocity_target_tick(self: &mut WinchController, cal: &WinchCalibration, m_per_s: f64) {
        let counts_per_tick = cal.dist_from_m(m_per_s) / (TICK_HZ as f64);
        let pos = self.fract_position_target + counts_per_tick;
        let fract = pos.fract();
        self.fract_position_target = fract;
        let int_diff = (pos - fract).round() as i32;
        self.quantized_position_target = self.quantized_position_target.wrapping_add(int_diff);
    }

    fn is_contiguous(self: &mut WinchController, tick_counter: u32) -> bool {
        match self.last_tick_counter {
            None => false,
            Some(last_tick_counter) => tick_counter.wrapping_sub(last_tick_counter) <= 2,
        }
    }

    fn reset(self: &mut WinchController, status: &WinchStatus) {
        // Initialize assumed winch state from this packet
        self.quantized_position_target = status.sensors.position;
        self.fract_position_target = 0.0;
    }

    fn update(self: &mut WinchController, config: &Config, id: usize, status: &WinchStatus) {
        if status.motor.pwm.enabled == 0 || !self.is_contiguous(status.tick_counter) {
            self.reset(status);
        }
        self.last_tick_counter = Some(status.tick_counter);
    }

    fn make_command(self: &mut WinchController, config: &Config, cal: &WinchCalibration, status: &WinchStatus) -> WinchCommand {
        WinchCommand {
            force: ForceCommand {
                filter_param: config.params.force_filter_param as f32,
                neg_motion_min: cal.force_from_kg(config.params.force_neg_motion_min_kg) as f32,
                pos_motion_max: cal.force_from_kg(config.params.force_pos_motion_max_kg) as f32,
                lockout_below: cal.force_from_kg(config.params.force_lockout_below_kg) as f32,
                lockout_above: cal.force_from_kg(config.params.force_lockout_above_kg) as f32,
            },
            pid: PIDGains {
                gain_p: cal.pwm_gain_from_m(config.params.pwm_gain_p) as f32,
                gain_i: cal.pwm_gain_from_m(config.params.pwm_gain_i) as f32,
                gain_d: cal.pwm_gain_from_m(config.params.pwm_gain_d) as f32,
                d_filter_param: config.params.vel_err_filter_param as f32,
            },
            position: match config.mode {

                ControllerMode::Halted => {
                    self.reset(status);
                    status.sensors.position
                },

                _ => self.quantized_position_target,
            }
        }
    }
}
