use vecmath::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LightEnvironment {
	pub winches: Vec<WinchLighting>,
	pub flash_exponent: f64,
	pub flash_rate_hz: f64,
	pub winch_wave_exponent: f64,
	pub winch_wavelength: f64,
	pub winch_wave_window_length: f64,
	pub winch_command_color: Vector3<f64>,
	pub winch_motion_color: Vector3<f64>,
	pub brightness: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WinchLighting {
	pub command_phase: f64,
	pub motion_phase: f64,
	pub wave_amplitude: f64,
	pub base_color: Vector3<f64>,
	pub flash_color: Vector3<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PixelMapping {
	pub location: Vector3<f64>,
	pub usage: PixelUsage,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PixelUsage {
	Winch(usize),
	FlyerRing,
	FlyerTop,
}

#[derive(Debug)]
pub struct Shader {
	pub flash_state_radians: f64,
}

fn pulse_wave(angle: f64, exponent: f64) -> f64 {
	(0.5 + 0.5 * angle.sin()).powf(exponent)
}

impl Shader {
	pub fn new() -> Shader {
		Shader {
			flash_state_radians: 0.0
		}
	}

	pub fn step(&mut self, env: &LightEnvironment, seconds: f64) {
		self.flash_state_radians = (self.flash_state_radians + seconds * env.flash_rate_hz * TAU) % TAU;
	}

	fn flash(&self, env: &LightEnvironment) -> f64 {
		pulse_wave(self.flash_state_radians, env.flash_exponent)
	}

	fn winch_wave(&self, winch: &WinchLighting, env: &LightEnvironment, map: &PixelMapping, phase: f64) -> f64 {
		let z = map.location[2];
		let pulse_theta = phase + z * TAU / env.winch_wavelength;
		let window_theta = z * TAU / env.winch_wave_window_length;
		let window = window_theta.min(PI).max(-PI).cos() * 0.5 + 0.5;
		winch.wave_amplitude * window * pulse_wave(pulse_theta, env.winch_wave_exponent)
	}

	pub fn pixel(&self, env: &LightEnvironment, map: &PixelMapping) -> Vector3<f64> {
		let c = match map.usage {

			PixelUsage::Winch(id) => {
				let winch = &env.winches[id];
				let c = winch.base_color;
				let c = vec3_mix(c, winch.flash_color, self.flash(env));
				let c = vec3_add(c, vec3_scale(env.winch_command_color, self.winch_wave(winch, env, map, winch.command_phase)));
				let c = vec3_add(c, vec3_scale(env.winch_motion_color, self.winch_wave(winch, env, map, winch.motion_phase)));
				c
			},

			PixelUsage::FlyerRing => {
				[0.3, 0.3, 0.3]
			},

			PixelUsage::FlyerTop => {
				[0.3, 0.3, 0.3]
			},

		};
		vec3_scale(c, env.brightness)
	}
}
