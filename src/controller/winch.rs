use bus::*;
use vecmath::*;
use config::{Config, ControllerMode, WinchCalibration};
use led::{WinchLighting, Rgb};

pub struct WinchController {
    id: usize,
    last_winch_status: Option<WinchStatus>,
    quantized_position_target: i32,
    fract_position_target: f64,
    lighting_command_phase: f64,
    lighting_motion_phase: f64,
    mech_status: MechStatus,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum MechStatus {
    Normal,
    ForceLimited(f64),
    Stuck,
}

impl MechStatus {
    fn update(&mut self, cal: &WinchCalibration, status: &WinchStatus) {
        let motor_off = status.motor.pwm.enabled == 0;
        let position_err = status.command.position.wrapping_sub(status.sensors.position);
        let outside_position_deadband = position_err.abs() > status.command.deadband.position;

        *self = if status.sensors.force.filtered > status.command.force.lockout_above {
            MechStatus::Stuck
        } else if status.sensors.force.filtered < status.command.force.lockout_below {
            MechStatus::Stuck
        } else if status.sensors.force.filtered > status.command.force.pos_motion_max {
            let counts = status.sensors.force.filtered - status.command.force.pos_motion_max;
            MechStatus::ForceLimited(counts as f64 * cal.kg_force_per_count)
        } else if status.sensors.force.filtered < status.command.force.neg_motion_min {
            let counts = status.sensors.force.filtered - status.command.force.neg_motion_min;
            MechStatus::ForceLimited(counts as f64 * cal.kg_force_per_count)
        } else if motor_off {
            if outside_position_deadband {
                // Motor off and we're trying to move, call it stuck
                MechStatus::Stuck
            } else {
                // Motor off but maybe it's fine, keep last value
                *self
            }
        } else {
            // Motor on, sensors in range
            MechStatus::Normal
        };
    }
}

fn lighting_motion(config: &Config, radians: f64, meters: f64) -> f64 {
    (meters * TAU / config.lighting.current.winch.wavelength_m + radians) % TAU
}

impl WinchController {
    pub fn new(id: usize) -> WinchController {
        WinchController {
            id,
            last_winch_status: None,
            quantized_position_target: 0,
            fract_position_target: 0.0,
            lighting_command_phase: 0.0,
            lighting_motion_phase: 0.0,
            mech_status: MechStatus::Normal,
        }
    }

    fn move_position_target(self: &mut WinchController, cal: &WinchCalibration, distance_m: f64) {
        let distance_counts = cal.dist_from_m(distance_m);
        let pos = self.fract_position_target + distance_counts;
        let fract = pos.fract();
        self.fract_position_target = fract;
        let int_diff = (pos - fract).round() as i32;
        self.quantized_position_target = self.quantized_position_target.wrapping_add(int_diff);
    }

    /// Apply one tick's worth of position change at the indicated velocity
    pub fn velocity_tick(self: &mut WinchController, config: &Config, cal: &WinchCalibration, m_per_s: f64) {
        let distance_m = m_per_s / (TICK_HZ as f64);
        self.move_position_target(cal, distance_m);
        self.lighting_command_phase = lighting_motion(config, self.lighting_command_phase, distance_m);
    }

    fn apply_sensed_motion(self: &mut WinchController, config: &Config, distance_m: f64) {
        self.lighting_motion_phase = lighting_motion(config, self.lighting_motion_phase, distance_m);
    }

    fn lighting_base_color(&self, config: &Config) -> Rgb {
        if self.mech_status != MechStatus::Normal {
            vec3_to_rgb(config.lighting.current.winch.error_colors.0)
        } else {
            match config.mode {
                ControllerMode::Halted => vec3_to_rgb(config.lighting.current.winch.halt_color),
                ControllerMode::ManualWinch(id) if id == self.id => vec3_to_rgb(config.lighting.current.winch.manual_color),
                _ => vec3_to_rgb(config.lighting.current.winch.normal_color),
            }
        }
    }

    fn lighting_flash_color(&self, config: &Config) -> Rgb {
        if self.mech_status == MechStatus::Stuck {
            vec3_to_rgb(config.lighting.current.winch.error_colors.1)
        } else {
            self.lighting_base_color(config)
        }
    }

    fn lighting_wave_amplitude(&self, config: &Config, cal: &WinchCalibration) -> f64 {
        match self.last_winch_status {
            None => 0.0,
            Some(ref status) => {
                let speed = cal.dist_to_m(status.sensors.velocity as f64).abs();
                let full_scale = config.lighting.current.winch.speed_for_full_wave_amplitude_m_per_sec;
                (speed / full_scale).min(1.0) * config.lighting.current.winch.wave_amplitude
            }
        }
    }

    pub fn light_environment(&self, config: &Config, cal: &WinchCalibration) -> WinchLighting {
        WinchLighting {
            command_phase: self.lighting_command_phase,
            motion_phase: self.lighting_motion_phase,
            wave_amplitude: self.lighting_wave_amplitude(config, cal),
            base_color: self.lighting_base_color(config),
            flash_color: self.lighting_flash_color(config),
        }
    }

    fn is_contiguous(self: &mut WinchController, tick_counter: u32) -> bool {
        match self.last_winch_status {
            None => false,
            Some(ref status) => tick_counter.wrapping_sub(status.tick_counter) <= 2,
        }
    }

    fn reset(self: &mut WinchController, status: &WinchStatus) {
        // Initialize assumed winch state from this packet
        self.last_winch_status = Some(status.clone());
        self.quantized_position_target = status.sensors.position;
        self.fract_position_target = 0.0;
    }

    pub fn update(self: &mut WinchController, config: &Config, cal: &WinchCalibration, status: &WinchStatus) {
        if status.motor.pwm.enabled == 0
            || !self.is_contiguous(status.tick_counter)
            || config.mode == ControllerMode::Halted {
            self.reset(status);
        }

        let distance_traveled_m = match self.last_winch_status {
            None => 0.0,
            Some(ref last_status) =>
                cal.dist_to_m(status.sensors.position.wrapping_sub(last_status.sensors.position) as f64)
        };
        self.apply_sensed_motion(config, distance_traveled_m);

        self.mech_status.update(cal, status);
        self.last_winch_status = Some(status.clone());
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
