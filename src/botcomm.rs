//! This module is about communicating with our many robot
//! modules via a custom UDP protocol.

use std::net::SocketAddr;
use cgmath::Vector3;

#[derive(Debug)]
pub struct WinchConfig {
	pub addr: SocketAddr,
	pub loc: Vector3<f64>,
}

#[derive(Debug)]
pub struct BotConfig {
	pub controller_addr: SocketAddr,
	pub flyer_addr: SocketAddr,
	pub winches: Vec<WinchConfig>,
}

