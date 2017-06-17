extern crate gilrs;
extern crate futures;
extern crate byteorder;
extern crate nom;
extern crate hyper;
extern crate websocket;
extern crate crc16;
extern crate cgmath;
extern crate tokio_proto;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_periodic;
extern crate palette;

mod botcomm;
mod fygimbal;
mod wscontrol;
pub mod leds;

pub use botcomm::{BotConfig, WinchConfig};
