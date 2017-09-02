use vecmath::*;
use palette;
use palette::Mix;

pub type Rgb = palette::Rgb<f64>;

#[derive(Clone, Debug, PartialEq)]
pub struct LightEnvironment {
	pub winches: Vec<WinchLighting>,
	pub flash_exponent: f64,
	pub flash_rate_hz: f64,
	pub winch_wave_exponent: f64,
	pub winch_wavelength: f64,
	pub winch_command_color: Rgb,
	pub winch_motion_color: Rgb,
	pub brightness: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WinchLighting {
	pub command_phase: f64,
	pub motion_phase: f64,
	pub wave_amplitude: f64,
	pub base_color: Rgb,
	pub flash_color: Rgb,
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
		let theta = phase + z * TAU / env.winch_wavelength;
		winch.wave_amplitude * pulse_wave(theta, env.winch_wave_exponent)
	}

	pub fn pixel(&self, env: &LightEnvironment, map: &PixelMapping) -> Rgb {
		let usage_specific = match map.usage {

			PixelUsage::Winch(id) => {
				let winch = &env.winches[id];
				winch.base_color
					.mix(&winch.flash_color, self.flash(env))
					+ env.winch_command_color * self.winch_wave(winch, env, map, winch.command_phase)
					+ env.winch_motion_color * self.winch_wave(winch, env, map, winch.motion_phase)
			},

			PixelUsage::FlyerRing => {
				Rgb::new(0.3, 0.3, 0.3)
			},

			PixelUsage::FlyerTop => {
				Rgb::new(0.3, 0.3, 0.3)
			},

		};
		usage_specific * env.brightness
	}
}
