use vecmath::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LightEnvironment {
	pub winches: Vec<WinchLighting>,
	pub flash_exponent: f32,
	pub flash_rate_hz: f32,
	pub winch_wave_exponent: f32,
	pub winch_wavelength: f32,
	pub winch_wave_window_length: f32,
	pub winch_command_color: Vector3<f32>,
	pub winch_motion_color: Vector3<f32>,
	pub brightness: f32,
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
	pub location: Vector3<f32>,
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
	pub flash_state_radians: f32,
	pub temp_spin_radians: f32,
}

fn pulse_wave(angle: f32, exponent: f32) -> f32 {
	(0.5 + 0.5 * angle.sin()).powf(exponent)
}

impl Shader {
	pub fn new() -> Shader {
		Shader {
			flash_state_radians: 0.0,
			temp_spin_radians: 0.0,
		}
	}

	pub fn step(&mut self, env: &LightEnvironment, seconds: f32) {
		self.flash_state_radians = (self.flash_state_radians + seconds * env.flash_rate_hz * TAU) % TAU;
		const SPIN_HZ : f32 = 0.5;
		self.temp_spin_radians = (self.temp_spin_radians + seconds * SPIN_HZ * TAU) % TAU;
	}

	fn flash(&self, env: &LightEnvironment) -> f32 {
		pulse_wave(self.flash_state_radians, env.flash_exponent)
	}

	fn winch_wave(&self, winch: &WinchLighting, env: &LightEnvironment, map: &PixelMapping, phase: f32) -> f32 {
		let z = map.location[2];
		let pulse_theta = phase + z * TAU / env.winch_wavelength;
		let window_theta = z * TAU / env.winch_wave_window_length;
		let window = window_theta.min(PI).max(-PI).cos() * 0.5 + 0.5;
		winch.wave_amplitude * window * pulse_wave(pulse_theta, env.winch_wave_exponent)
	}

	pub fn pixel(&self, env: &LightEnvironment, map: &PixelMapping) -> Vector3<f32> {
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
				[0.1, 0.1, 0.1]
			},

			PixelUsage::FlyerTop => {
				[0.1, 0.1, 0.1]
			},

			// _ => {
			// 	let spin_vec = [ self.temp_spin_radians.cos(), self.temp_spin_radians.sin() ];
			// 	let xy_vec = vec2_normalized([ map.location[0], map.location[1] ]);
			// 	let blip = vec2_dot(spin_vec, xy_vec).powi(15);
			// 	let color_z = (map.location[2] * 10.0) % 1.0;
			// 	let color_z = if color_z < 0.0 { color_z + 1.0 } else { color_z };
			// 	let color = vec3_mix([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], color_z);
			// 	let result = vec3_scale(color, blip);
			// 	result
			// }

		};
		vec3_scale(c, env.brightness)
	}
}
