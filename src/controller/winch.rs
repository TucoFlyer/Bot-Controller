use message::*;
use vecmath::*;
use std::time::{Duration, Instant};
use config::{Config, ControllerMode, WinchCalibration};
use led::WinchLighting;

pub struct WinchController {
    pub mech_status: MechStatus,
    id: usize,
    last_winch_status: Option<(WinchStatus, Instant)>,
    quantized_position_target: i32,
    fract_position_target: f32,
    pwm_period: f32,
    lighting_command_phase: f32,
    lighting_motion_phase: f32,
    lighting_filtered_velocity: f32,
    has_nonzero_velocity_command: bool,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MechStatus {
    Normal,
    /// How far into force limit, always in [-1, +1]
    ForceLimited(f32),
    Stuck,
}

impl WinchController {
    pub fn new(id: usize) -> WinchController {
        WinchController {
            id,
            last_winch_status: None,
            quantized_position_target: 0,
            fract_position_target: 0.0,
            pwm_period: 0.0,
            lighting_command_phase: 0.0,
            lighting_motion_phase: 0.0,
            lighting_filtered_velocity: 0.0,
            mech_status: MechStatus::Normal,
            has_nonzero_velocity_command: false,
        }
    }

    /// Apply one tick's worth of position change at the indicated velocity
    pub fn motion_tick_with_velocity(&mut self, config: &Config, cal: &WinchCalibration, m_per_s: f32) {
        let distance_m = m_per_s / (TICK_HZ as f32);
        self.move_position_target(cal, distance_m);
        self.lighting_command_phase += distance_m * TAU / config.lighting.current.winch.wavelength_m;
        self.has_nonzero_velocity_command = m_per_s != 0.0;
    }

    pub fn light_environment(&self, config: &Config) -> WinchLighting {
        WinchLighting {
            command_phase: self.lighting_command_phase,
            motion_phase: self.lighting_motion_phase,
            wave_amplitude: self.lighting_wave_amplitude(config),
            base_color: self.lighting_base_color(config),
            flash_color: self.lighting_flash_color(config),
        }
    }

    pub fn update(&mut self, config: &Config, cal: &WinchCalibration, status: &WinchStatus) {
        if config.mode == ControllerMode::Halted
            || self.was_motor_shutoff(status)
            || self.was_tick_discontinuity(status) {
            self.reset(status);
        }

        let distance_traveled_m = match self.last_winch_status {
            None => 0.0,
            Some((ref last_status, _)) =>
                cal.dist_to_m(status.sensors.position.wrapping_sub(last_status.sensors.position) as f32)
        };
        self.apply_sensed_motion(config, distance_traveled_m);

        let velocity_m = cal.dist_to_m(status.sensors.velocity as f32);
        let velocity_filter_param = config.lighting.current.winch.velocity_filter_param;
        self.lighting_filtered_velocity += (velocity_m - self.lighting_filtered_velocity) * velocity_filter_param;

        let pwm_period_high_motion = 1.0 / config.params.pwm_hz_high_motion;
        let pwm_period_low_motion = 1.0 / config.params.pwm_hz_low_motion;
        let pwm_period_min = pwm_period_high_motion.min(pwm_period_low_motion);
        let pwm_period_max = pwm_period_high_motion.max(pwm_period_low_motion);
        let high_speed = velocity_m.abs() >= config.params.pwm_velocity_threshold;
        let pwm_period_target = if high_speed { pwm_period_high_motion } else { pwm_period_low_motion };
        self.pwm_period += (pwm_period_target - self.pwm_period) * config.params.pwm_hz_filter_param;
        self.pwm_period = self.pwm_period.max(pwm_period_min).min(pwm_period_max);

        self.mech_status = MechStatus::new(status);
        self.last_winch_status = Some((status.clone(), Instant::now()));
    }

    pub fn make_command(&self, config: &Config, cal: &WinchCalibration, status: &WinchStatus) -> WinchCommand {
        match config.mode {

            ControllerMode::Halted => WinchCommand {
                position: status.sensors.position,
                force: self.make_force_command(config, cal),
                pid: self.make_halted_pid_gains(),
                deadband: self.make_deadband(config, cal),
                pwm: self.make_pwm_command(config),
            },

            _ => WinchCommand {
                position: self.quantized_position_target,
                force: self.make_force_command(config, cal),
                pid: self.make_pid_gains(config, cal),
                deadband: self.make_deadband(config, cal),
                pwm: self.make_pwm_command(config),
            },
        }
    }

    fn move_position_target(&mut self, cal: &WinchCalibration, distance_m: f32) {
        let distance_counts = cal.dist_from_m(distance_m);
        let pos = self.fract_position_target + distance_counts;
        let fract = pos.fract();
        self.fract_position_target = fract;
        let int_diff = (pos - fract).round() as i32;
        self.quantized_position_target = self.quantized_position_target.wrapping_add(int_diff);
    }

    fn apply_sensed_motion(self: &mut WinchController, config: &Config, distance_m: f32) {
        self.lighting_motion_phase += distance_m * TAU / config.lighting.current.winch.wavelength_m;
    }

    fn lighting_base_color(&self, config: &Config) -> Vector3<f32> {
        if self.mech_status != MechStatus::Normal {
            config.lighting.current.winch.error_color
        } else {
            match config.mode {
                ControllerMode::ManualWinch(id) => {
                    if id == self.id {
                        config.lighting.current.winch.manual_selected_color
                    } else {
                        config.lighting.current.winch.manual_deselected_color
                    }
                },
                ControllerMode::Halted => config.lighting.current.winch.halt_color,
                _ => config.lighting.current.winch.normal_color,
            }
        }
    }

    fn lighting_flash_color(&self, config: &Config) -> Vector3<f32> {
        if self.mech_status == MechStatus::Stuck {
            config.lighting.current.winch.stuck_color
        } else {
            self.lighting_base_color(config)
        }
    }

    fn lighting_wave_amplitude(&self, config: &Config) -> f32 {
        match config.mode {
            ControllerMode::Halted => 0.0,
            _ => if self.mech_status != MechStatus::Normal {
                0.0
            } else {
                let full_scale = config.lighting.current.winch.speed_for_full_wave_amplitude_m_per_sec;
                (self.lighting_filtered_velocity.abs() / full_scale).min(1.0) * config.lighting.current.winch.wave_amplitude
            }
        }
    }

    fn was_motor_shutoff(&self, status: &WinchStatus) -> bool {
        match self.last_winch_status {
            None => status.motor.pwm.enabled == 0,
            Some((ref last_status, _)) => last_status.motor.pwm.enabled != 0 && status.motor.pwm.enabled == 0
        }
    }

    fn was_tick_discontinuity(&self, status: &WinchStatus) -> bool {
        match self.last_winch_status {
            None => true,
            Some((ref last_status, _)) => status.tick_counter.wrapping_sub(last_status.tick_counter) > 2,
        }
    }

    pub fn is_status_recent(&self, config: &Config) -> bool {
        let deadline = Duration::from_millis(config.params.winch_watchdog_millis);
        match self.last_winch_status {
            None => false,
            Some((_, timestamp)) => timestamp + deadline > Instant::now(),
        }
    }

    fn reset(&mut self, status: &WinchStatus) {
        // Initialize assumed winch state from this packet
        self.last_winch_status = Some((status.clone(), Instant::now()));
        self.quantized_position_target = status.sensors.position;
        self.fract_position_target = 0.0;
    }

    fn make_force_command(&self, config: &Config, cal: &WinchCalibration) -> ForceCommand {
        ForceCommand {
            filter_param: config.params.force_filter_param as f32,
            neg_motion_min: cal.force_from_kg(config.params.force_neg_motion_min_kg) as f32,
            pos_motion_max: cal.force_from_kg(config.params.force_pos_motion_max_kg) as f32,
            lockout_below: cal.force_from_kg(config.params.force_lockout_below_kg) as f32,
            lockout_above: cal.force_from_kg(config.params.force_lockout_above_kg) as f32,
        }
    }

    fn make_pwm_command(&self, config: &Config) -> WinchPWMCommand {
        let hz = if self.pwm_period > 0.0 { 1.0 / self.pwm_period } else { 0.0 };
        WinchPWMCommand {
            hz,
            bias: config.params.pwm_bias,
            minimum: config.params.pwm_minimum,
        }
    }

    fn make_pid_gains(&self, config: &Config, cal: &WinchCalibration) -> PIDGains {
        PIDGains {
            gain_p: cal.pwm_gain_from_m(config.params.pwm_gain_p) as f32,
            gain_i: cal.pwm_gain_from_m(config.params.pwm_gain_i) as f32,
            gain_d: cal.pwm_gain_from_m(config.params.pwm_gain_d) as f32,
            p_filter_param: config.params.pos_err_filter_param as f32,
            i_decay_param: config.params.integral_err_decay_param as f32,
            d_filter_param: config.params.vel_err_filter_param as f32,
        }
    }

    fn make_halted_pid_gains(&self) -> PIDGains {
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

    fn make_deadband(&self, config: &Config, cal: &WinchCalibration) -> WinchDeadband {
        WinchDeadband {
            velocity: cal.dist_from_m(config.params.deadband_velocity_limit_m_per_sec) as f32,
            position: if self.has_nonzero_velocity_command {
                // Intending to move; send a zero to disable the firmware deadband support
                0
            } else {
                // Done with commanded move; use configured deadband for position/velocity
                // to finish moving once the PID loop has mostly converged.
                cal.dist_from_m(config.params.deadband_position_err_m).round() as i32
            },
        }
    }
}

impl MechStatus {
    fn new(status: &WinchStatus) -> MechStatus {
        let motor_off = status.motor.pwm.enabled == 0;
        let position_err = status.command.position.wrapping_sub(status.sensors.position);
        let outside_position_deadband = position_err.abs() > status.command.deadband.position;

        // Force lockout and we're stuck
        if status.sensors.force.filtered > status.command.force.lockout_above {
            MechStatus::Stuck
        } else if status.sensors.force.filtered < status.command.force.lockout_below {
            MechStatus::Stuck

        // Force limited, can't move any further
        } else if status.sensors.force.filtered > status.command.force.pos_motion_max {
            let counts = status.sensors.force.filtered - status.command.force.pos_motion_max;
            let unit_value = status.command.force.lockout_above - status.command.force.pos_motion_max;
            MechStatus::ForceLimited(counts as f32 / unit_value.max(1.0))
        } else if status.sensors.force.filtered < status.command.force.neg_motion_min {
            let counts = status.sensors.force.filtered - status.command.force.neg_motion_min;
            let unit_value = status.command.force.neg_motion_min - status.command.force.lockout_below;
            MechStatus::ForceLimited(counts as f32 / unit_value.max(1.0))

        } else if motor_off {
            if outside_position_deadband {
                // Motor off and we're trying to move, call it stuck
                MechStatus::Stuck
            } else {
                // Motor off but no big deal
                MechStatus::Normal
            }
        } else {
            // Motor on, sensors in range
            MechStatus::Normal
        }
    }
}
