
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate multiqueue;
extern crate bincode;
extern crate gilrs;
extern crate palette;
extern crate crc16;
extern crate cgmath;
extern crate byteorder;
extern crate nom;

extern crate hyper;
extern crate websocket;
extern crate futures;
extern crate tokio_proto;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_periodic;

mod fygimbal;
mod leds;

mod bus;
pub use bus::*;

pub mod interface;
pub mod controller;
pub mod botcomm;

mod config;
pub use config::{BotConfig, WinchConfig};
