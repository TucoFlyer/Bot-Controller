use vecmath::*;
use palette;
use config::{Config, ControllerMode};

pub type Rgb = palette::Rgb<f64>;

#[derive(Clone, Debug, PartialEq)]
pub struct LightEnvironment {
	pub rgb_brightness_scale: Rgb,
	pub ctrl_mode: ControllerMode,
	pub winches: Vec<WinchLighting>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WinchLighting {
	pub velocity_m_per_sec: f64,
	pub wavelength_m: f64,
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
	winches: Vec<WinchState>,
}

#[derive(Debug)]
struct WinchState {
	position_mod_tau: f64,
}

impl WinchLighting {
	fn m_to_radians(&self, meters: f64) -> f64 {
		meters * TAU / self.wavelength_m
	}
}

impl Shader {
	pub fn new(config: &Config) -> Shader {
		let winches = config.winches.iter().map(|_config| {
			WinchState {
				position_mod_tau: 0.0
			}
		}).collect();
		Shader { winches }
	}

	pub fn step(&mut self, env: &LightEnvironment, seconds: f64) {
		for (id, winch_env) in env.winches.iter().enumerate() {
			let radians_per_sec = winch_env.m_to_radians(winch_env.velocity_m_per_sec);
			let position = self.winches[id].position_mod_tau + radians_per_sec * seconds;
			self.winches[id].position_mod_tau = position % TAU;
		}
	}

	pub fn pixel(&self, env: &LightEnvironment, map: &PixelMapping) -> Rgb {
		let usage_specific = match map.usage {

			PixelUsage::Winch(id) => {
				let winch_state = &self.winches[id];
				let winch_env = &env.winches[id];

				let base_color = Rgb::new(0.0, 0.0, 0.0);
				let wave_color = Rgb::new(0.6, 0.6, 0.6);

				let y_axis = map.location[1];
				let displaced_position = winch_state.position_mod_tau + winch_env.m_to_radians(y_axis);
				base_color + wave_color * displaced_position.sin()
			},

			PixelUsage::FlyerRing => {
				Rgb::new(0.3, 0.3, 0.3)
			},

			PixelUsage::FlyerTop => {
				Rgb::new(0.3, 0.3, 0.3)
			},

		};
		usage_specific * env.rgb_brightness_scale
	}
}
