extern crate rand;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate bincode;

extern crate multiqueue;

extern crate gilrs;
extern crate palette;
extern crate crc16;
extern crate cgmath;
extern crate byteorder;
extern crate nom;

extern crate hyper;
extern crate websocket;
extern crate qrcode;

mod fygimbal;
mod leds;

mod bus;
pub use bus::*;

pub mod interface;
pub mod controller;
pub mod botcomm;
pub mod watchdog;

mod config;
pub use config::{BotConfig, WinchConfig, WebConfig};
