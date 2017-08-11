//! Bot configuration

use std::net::{SocketAddr, UdpSocket};
use cgmath::Point3;


#[derive(Clone, PartialEq, Debug)]
pub struct WinchConfig {
	pub addr: SocketAddr,
	pub loc: Point3<f64>
}

#[derive(Clone, PartialEq, Debug)]
pub struct BotConfig {
	pub controller_addr: SocketAddr,
	pub flyer_addr: SocketAddr,
	pub winches: Vec<WinchConfig>
}
