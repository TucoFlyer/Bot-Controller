use bus::*;
use config::{Config, ControllerMode, WinchCalibration};

pub struct WinchController {
    last_tick_counter: Option<u32>,
    quantized_position_target: i32,
    fract_position_target: f64,
}

impl WinchController {
    pub fn new() -> WinchController {
        WinchController {
            last_tick_counter: None,
            quantized_position_target: 0,
            fract_position_target: 0.0
        }
    }

    pub fn velocity_tick(self: &mut WinchController, cal: &WinchCalibration, m_per_s: f64) {
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

    pub fn update(self: &mut WinchController, config: &Config, status: &WinchStatus) {
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
            p_filter_param: config.params.pos_err_filter_param as f32,
            i_decay_param: config.params.integral_err_decay_param as f32,
            d_filter_param: config.params.vel_err_filter_param as f32,
        }   
    }

    fn make_halted_pid_gains(self: &WinchController) -> PIDGains {
        // Disable the PID entirely when halted
        PIDGains {
            gain_p: 0.0,
            gain_i: 0.0,
            gain_d: 0.0,
            p_filter_param: 1.0,
            i_decay_param: 1.0,
            d_filter_param: 1.0,
        }
    }
  
    fn make_deadband(self: &WinchController, config: &Config, cal: &WinchCalibration) -> WinchDeadband {
        WinchDeadband {
            position: cal.dist_from_m(config.params.deadband_position_err_m).round() as i32,
            velocity: cal.dist_from_m(config.params.deadband_velocity_limit_m_per_sec) as f32,
        }
    }

    pub fn make_command(self: &WinchController, config: &Config, cal: &WinchCalibration, status: &WinchStatus) -> WinchCommand {
        match config.mode {

            ControllerMode::Halted => WinchCommand {
                position: status.sensors.position,
                force: self.make_force_command(config, cal),
                pid: self.make_halted_pid_gains(),
                deadband: self.make_deadband(config, cal),
            },

            _ => WinchCommand {
                position: self.quantized_position_target,
                force: self.make_force_command(config, cal),
                pid: self.make_pid_gains(config, cal),
                deadband: self.make_deadband(config, cal),
            },
        }
    }
}
