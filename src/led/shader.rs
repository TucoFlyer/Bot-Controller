use vecmath::*;
use config::LightingScheme;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LightEnvironment {
	pub config: LightingScheme,
	pub winches: Vec<WinchLighting>,
	pub ring_color: Vector3<f32>,
	pub camera_yaw_angle: f32,
	pub is_streaming: bool,
	pub is_recording: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WinchLighting {
	pub command_phase: f32,
	pub motion_phase: f32,
	pub wave_amplitude: f32,
	pub base_color: Vector3<f32>,
	pub flash_color: Vector3<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PixelMapping {
	pub format: Vector3<u8>,
	pub usage: PixelUsage,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PixelUsage {
	Winch(usize, f32, f32),
	FlyerSaucer(usize, f32),
	FlyerRing(f32, f32),
}

#[derive(Debug)]
pub struct Shader {
	flash_state_radians: f32,
	dot_pattern_buffer: Vec<i8>,
	dot_pattern_timer: f32,
	dot_output_filter: f32,
}

fn pulse_wave(angle: f32, exponent: f32) -> f32 {
	(0.5 + 0.5 * angle.sin()).powf(exponent)
}

fn cos_single(angle: f32) -> f32 {
	angle.max(-PI/2.0).min(PI/2.0).cos()
}

impl Shader {
	pub fn new() -> Shader {
		Shader {
			flash_state_radians: 0.0,
			dot_pattern_buffer: Vec::new(),
			dot_pattern_timer: 0.0,
			dot_output_filter: 0.0,
		}
	}

	pub fn step(&mut self, env: &LightEnvironment, seconds: f32) {
		self.flash_state_radians = (self.flash_state_radians + seconds * env.config.flash_rate_hz * TAU) % TAU;

		// Keep the dot pattern buffer full
		if self.dot_pattern_buffer.len() < 1 {
			self.dot_pattern_buffer = env.config.flyer_dot_pattern_base.clone();
			if env.is_streaming {
				self.dot_pattern_buffer.extend(env.config.flyer_dot_pattern_is_streaming.iter());
			}
			if env.is_recording {
				self.dot_pattern_buffer.extend(env.config.flyer_dot_pattern_is_recording.iter());
			}
		}

		// Filter the dot flashing pattern each tick. Sign of pattern indicates mark vs space.
		let dot_target = if self.dot_pattern_buffer[0] > 0 { 1.0 } else { 0.0 };
		self.dot_output_filter += (dot_target - self.dot_output_filter) * (1.0 - env.config.flyer_dot_pattern_smoothness);

		// Time the changeover from dot to dot
		let dot_period = 1.0 / env.config.flyer_dot_pattern_rate.max(1e-4);
		let dot_pattern_timer = self.dot_pattern_timer + seconds;
		if dot_pattern_timer > dot_period {
			if self.dot_pattern_buffer[0] > 1 {
				self.dot_pattern_buffer[0] -= 1;
			} else if self.dot_pattern_buffer[0] < -1 {
				self.dot_pattern_buffer[0] += 1;
			} else {
				self.dot_pattern_buffer.remove(0);
			}
		}
		self.dot_pattern_timer = dot_pattern_timer % dot_period;
	}

	fn flash(&self, env: &LightEnvironment) -> f32 {
		pulse_wave(self.flash_state_radians, env.config.flash_exponent)
	}

	fn winch_wave(&self, winch: &WinchLighting, env: &LightEnvironment, loc: f32, phase: f32) -> f32 {
		let pulse_theta = phase + loc * TAU / env.config.winch_wavelength_m;
		let window_theta = loc * TAU / env.config.winch_wave_window_length_m;
		let window = window_theta.min(PI).max(-PI).cos() * 0.5 + 0.5;
		winch.wave_amplitude * window * pulse_wave(pulse_theta, env.config.winch_wave_exponent)
	}

	fn winch_pixel(&self, winch: &WinchLighting, env: &LightEnvironment, loc: f32) -> Vector3<f32> {
		let c = winch.base_color;
		let c = vec3_mix(c, winch.flash_color, self.flash(env));
		let c = vec3_add(c, vec3_scale(env.config.winch_command_color, self.winch_wave(winch, env, loc, winch.command_phase)));
		let c = vec3_add(c, vec3_scale(env.config.winch_motion_color, self.winch_wave(winch, env, loc, winch.motion_phase)));
		c
	}

	pub fn pixel(&self, env: &LightEnvironment, mapping: &PixelMapping) -> Vector3<f32> {
		let c = match &mapping.usage {

			&PixelUsage::Winch(id, loc, _x) => {
				self.winch_pixel(&env.winches[id], env, loc)
			},

			&PixelUsage::FlyerSaucer(id, loc) => {
				let c = self.winch_pixel(&env.winches[id], env, loc);
				vec3_scale(c, env.config.flyer_saucer_brightness)
			}

			&PixelUsage::FlyerRing(angle, z) => {
				let x = angle_normalize(angle - env.camera_yaw_angle);
				let z = z * env.config.flyer_z_scale;
				let distance_from_center = (x*x + z*z).sqrt();

				let dot = cos_single(distance_from_center * PI / env.config.flyer_dot_size);
				let dot_color = vec3_scale(env.config.flyer_dot_color, dot * self.dot_output_filter);

				let ring = cos_single((distance_from_center - env.config.flyer_ring_size) * PI / env.config.flyer_ring_thickness);

				let c = env.config.flyer_background_color;
				let c = vec3_mix(c, env.ring_color, ring);
				let c = vec3_mix(c, dot_color, dot);
				c
			},

		};
		vec3_scale(c, env.config.brightness)
	}
}
