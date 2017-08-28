//! Controller thread, responding to commands and status updates, generating motor control packets

use bus::*;
use vecmath::*;
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
                    self.state.manual.control_value(axis, value);
                },

                Message::Command(Command::ManualControlReset) => {
                    self.state.manual.control_reset();
                },

                _ => (),
            }
        }
    }
}


struct RateLimitedVelocity {
    vec: Vector3<f64>
}

impl RateLimitedVelocity {
    fn new() -> RateLimitedVelocity {
        RateLimitedVelocity {
            vec: [0.0, 0.0, 0.0]
        }
    }

    fn tick(self: &mut RateLimitedVelocity, config: &Config, target: Vector3<f64>) {
        let dt = 1.0 / (TICK_HZ as f64);
        let limit_per_tick = config.params.accel_limit_m_per_sec2 * dt;
        let diff = vec3_sub(target, self.vec);
        let len = vec3_len(diff);
        let clipped = if len > limit_per_tick {
            vec3_scale(diff, limit_per_tick / len)
        } else {
            diff
        };
        self.vec = vec3_add(self.vec, clipped);
    }
}

struct ManualControls {
    axes: HashMap<ManualControlAxis, f64>,
    velocity: RateLimitedVelocity,
}

impl ManualControls {
    fn new() -> ManualControls {
        ManualControls {
            axes: HashMap::new(),
            velocity: RateLimitedVelocity::new(),
        }
    }

    fn lookup_axis(self: &mut ManualControls, axis: ManualControlAxis) -> f64 {
        self.axes.entry(axis).or_insert(0.0).min(1.0).max(-1.0)
    }

    fn lookup_relative_vec(self: &mut ManualControls) -> Vector3<f64> {
        [
            self.lookup_axis(ManualControlAxis::RelativeX),
            self.lookup_axis(ManualControlAxis::RelativeY),
            self.lookup_axis(ManualControlAxis::RelativeZ),
        ]
    }

    fn velocity_target(self: &mut ManualControls, config: &Config) -> Vector3<f64> {
        let velocity_scale = config.params.manual_control_velocity_m_per_sec;
        vec3_scale(self.lookup_relative_vec(), velocity_scale)
    }

    fn control_tick(self: &mut ManualControls, config: &Config) {
        let v = self.velocity_target(config);
        self.velocity.tick(config, v);
    }

    fn control_value(self: &mut ManualControls, axis: ManualControlAxis, value: f64) {
        self.axes.insert(axis, value);
    }

    fn control_reset(self: &mut ManualControls) {
        self.axes.clear();
    }
}

struct ControllerState {
    manual: ManualControls,
    winches: Vec<WinchController>,
}

impl ControllerState {
    fn new(config: &Config) -> ControllerState {
        ControllerState {
            manual: ManualControls::new(),
            winches: config.winches.iter().map( |_| WinchController::new() ).collect(),
        }
    }

    fn flyer_sensor_update(self: &mut ControllerState, sensors: FlyerSensors) {

    }

    fn winch_control_loop(self: &mut ControllerState, config: &Config, id: usize, status: WinchStatus) -> WinchCommand {
        let cal = &config.winches[id].calibration;
        self.winches[id].update(config, id, &status);

        let velocity = match config.mode {

            ControllerMode::ManualWinch(manual_id) => {
                if manual_id == id {
                    self.manual.control_tick(config);
                    self.manual.velocity.vec[1]
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

    fn velocity_tick(self: &mut WinchController, cal: &WinchCalibration, m_per_s: f64) {
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
        if status.motor.pwm.enabled == 0
            || !self.is_contiguous(status.tick_counter)
            || config.mode == ControllerMode::Halted {
            self.reset(status);
        }
        self.last_tick_counter = Some(status.tick_counter);
    }

    fn make_force_command(self: &WinchController, config: &Config, cal: &WinchCalibration) -> ForceCommand {
        ForceCommand {
            filter_param: config.params.force_filter_param as f32,
            neg_motion_min: cal.force_from_kg(config.params.force_neg_motion_min_kg) as f32,
            pos_motion_max: cal.force_from_kg(config.params.force_pos_motion_max_kg) as f32,
            lockout_below: cal.force_from_kg(config.params.force_lockout_below_kg) as f32,
            lockout_above: cal.force_from_kg(config.params.force_lockout_above_kg) as f32,
        }
    }

    fn make_pid_gains(self: &WinchController, config: &Config, cal: &WinchCalibration) -> PIDGains {
        PIDGains {
            gain_p: cal.pwm_gain_from_m(config.params.pwm_gain_p) as f32,
            gain_i: cal.pwm_gain_from_m(config.params.pwm_gain_i) as f32,
            gain_d: cal.pwm_gain_from_m(config.params.pwm_gain_d) as f32,
            d_filter_param: config.params.vel_err_filter_param as f32,
            i_decay_param: config.params.integral_err_decay_param as f32,
        }   
    }

    fn make_halted_pid_gains(self: &WinchController) -> PIDGains {
        PIDGains {
            gain_p: 0.0,
            gain_i: 0.0,
            gain_d: 0.0,
            d_filter_param: 1.0,
            i_decay_param: 1.0,
        }
    }

    fn make_command(self: &WinchController, config: &Config, cal: &WinchCalibration, status: &WinchStatus) -> WinchCommand {
        match config.mode {

            ControllerMode::Halted => WinchCommand {
                force: self.make_force_command(config, cal),
                pid: self.make_halted_pid_gains(),
                position: status.sensors.position,
            },

            _ => WinchCommand {
                force: self.make_force_command(config, cal),
                pid: self.make_pid_gains(config, cal),
                position: self.quantized_position_target,
            },
        }
    }
}
