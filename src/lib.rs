// Auth related crypto
extern crate rand;
extern crate hmac;
extern crate sha2;
extern crate base64;

// All the serialization
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_yaml;
extern crate bincode;
extern crate atomicwrites;

// Bus parts
extern crate multiqueue;

// Gamepad input
extern crate gilrs;

// For LED colors
extern crate palette;

// For fygimbal
extern crate crc16;
extern crate byteorder;
extern crate nom;

// For the web interface
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate websocket;
extern crate qrcode;

extern crate vecmath as vecmath_lib;
mod vecmath;

mod fygimbal;
mod led;

mod bus;
pub use bus::*;

mod config;
pub use config::*;

mod botcomm;
pub use botcomm::BotComm;

pub mod interface;
pub mod controller;
pub mod watchdog;
