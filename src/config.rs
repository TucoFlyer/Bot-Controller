//! Bot configuration

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Config {
	pub bot: BotConfig,
	pub web: WebConfig,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BotConfig {
    pub controller_addr: SocketAddr,
    pub flyer_addr: SocketAddr,
    pub winches: Vec<WinchConfig>,
    pub params: BotParams,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WinchConfig {
    pub addr: SocketAddr,
    pub loc: Point3,
    pub calibration: WinchCalibration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Point3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl Point3 {
	pub fn new(x: f64, y: f64, z: f64) -> Point3 {
		Point3 { x, y, z }
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WinchCalibration {
	pub kg_force_at_zero: f64,
	pub kg_force_per_count: f64,
	pub m_dist_per_count: f64,
}

impl WinchCalibration {
	pub fn force_to_kg(self: &WinchCalibration, counts: f64) -> f64 {
		self.kg_force_at_zero + self.kg_force_per_count * counts
	}

	pub fn force_from_kg(self: &WinchCalibration, kg: f64) -> f64 {
		(kg - self.kg_force_at_zero) / self.kg_force_per_count
	}

	pub fn dist_to_m(self: &WinchCalibration, counts: f64) -> f64 {
		self.m_dist_per_count * counts
	}

	pub fn dist_from_m(self: &WinchCalibration, m: f64) -> f64 {
		m * self.m_dist_per_count
	}
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BotParams {
	pub accel_rate_m_per_sec2: f64,
	pub manual_control_velocity_m_per_sec: f64,
	pub force_min_kg: f64,
	pub force_max_kg: f64,
	pub force_filter_param: f32,
	pub pwm_gain_p: f32,
	pub pwm_gain_i: f32,
	pub pwm_gain_d: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WebConfig {
    pub http_addr: SocketAddr,
    pub ws_addr: SocketAddr,
    pub web_root_path: String,
    pub connection_file_path: String,
}

fn all_if_addr() -> IpAddr {
    // Bind to all interfaces; we need at least localhost and the LAN
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

impl WebConfig {
	pub fn http_bind_addr(self: &WebConfig) -> SocketAddr {
		SocketAddr::new(all_if_addr(), self.http_addr.port())
	}

	pub fn ws_bind_addr(self: &WebConfig) -> SocketAddr {
		SocketAddr::new(all_if_addr(), self.ws_addr.port())
	}

	pub fn http_uri(self: &WebConfig, secret_key: &str) -> String {
	    format!("http://{}/#?k={}", self.http_addr, secret_key)
	}

	pub fn ws_uri(self: &WebConfig) -> String {
	    format!("ws://{}", self.ws_addr)
	}
}

