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
	pub camera_yaw_angle: f32,
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
	pub format: Vector3<u8>,
	pub usage: PixelUsage,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PixelUsage {
	Winch(usize, f32, f32),
	FlyerRing(f32, f32),
}

#[derive(Debug)]
pub struct Shader {
	pub flash_state_radians: f32,
}

fn pulse_wave(angle: f32, exponent: f32) -> f32 {
	(0.5 + 0.5 * angle.sin()).powf(exponent)
}

impl Shader {
	pub fn new() -> Shader {
		Shader {
			flash_state_radians: 0.0,
		}
	}

	pub fn step(&mut self, env: &LightEnvironment, seconds: f32) {
		self.flash_state_radians = (self.flash_state_radians + seconds * env.flash_rate_hz * TAU) % TAU;
	}

	fn flash(&self, env: &LightEnvironment) -> f32 {
		pulse_wave(self.flash_state_radians, env.flash_exponent)
	}

	fn winch_wave(&self, winch: &WinchLighting, env: &LightEnvironment, loc: f32, phase: f32) -> f32 {
		let pulse_theta = phase + loc * TAU / env.winch_wavelength;
		let window_theta = loc * TAU / env.winch_wave_window_length;
		let window = window_theta.min(PI).max(-PI).cos() * 0.5 + 0.5;
		winch.wave_amplitude * window * pulse_wave(pulse_theta, env.winch_wave_exponent)
	}

	pub fn pixel(&self, env: &LightEnvironment, mapping: &PixelMapping) -> Vector3<f32> {
		let c = match &mapping.usage {

			&PixelUsage::Winch(id, loc, _x) => {
				let winch = &env.winches[id];
				let c = winch.base_color;
				let c = vec3_mix(c, winch.flash_color, self.flash(env));
				let c = vec3_add(c, vec3_scale(env.winch_command_color, self.winch_wave(winch, env, loc, winch.command_phase)));
				let c = vec3_add(c, vec3_scale(env.winch_motion_color, self.winch_wave(winch, env, loc, winch.motion_phase)));
				c
			},

			&PixelUsage::FlyerRing(angle, _z) => {
				let angle_diff = (angle - env.camera_yaw_angle) % TAU;
				let angle_diff = if angle_diff > PI { angle_diff - TAU } else { angle_diff };
				let angle_diff = if angle_diff < -PI { angle_diff + TAU } else { angle_diff };
				let blip = (angle_diff * 4.0).max(-PI/2.0).min(PI/2.0).cos();
				vec3_scale([0.2, 0.1, 0.1], blip)
			},

		};
		vec3_scale(c, env.brightness)
	}
}
