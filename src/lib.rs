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
extern crate bus;

// Metrics
extern crate dipstick;
#[macro_use]
extern crate lazy_static;

// Gamepad input
extern crate gilrs;

extern crate num;

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

mod message;
pub use message::*;

mod config;
pub use config::*;

mod controller;
pub use controller::{Controller, ControllerPort};

mod botcomm;
pub use botcomm::BotSocket;

pub mod interface;
pub mod watchdog;
