//! Bot configuration

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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

#[derive(Clone, PartialEq, Debug)]
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
