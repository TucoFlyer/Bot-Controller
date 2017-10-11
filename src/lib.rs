// Auth related crypto
extern crate rand;
extern crate hmac;
extern crate sha2;
extern crate base64;

// All the serialization
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate bincode;
extern crate atomicwrites;

extern crate chrono;

extern crate bmfont;

// Bus parts
extern crate multiqueue;

// Gamepad input
extern crate gilrs;

// For fygimbal
extern crate crc16;
extern crate byteorder;

// For the web interface
extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate websocket;
extern crate qrcode;
extern crate open;

extern crate vecmath as vecmath_lib;
mod vecmath;

mod fygimbal;
mod led;
mod overlay;

mod bus;
pub use bus::*;

mod config;
pub use config::*;

pub mod interface;
pub mod controller;
pub mod watchdog;
pub mod botcomm;
