//! This module is about communicating with our many robot
//! modules via a custom UDP protocol.

use bus::{Bus, Message, WinchCommand};
use std::thread;
use bincode;
use config::Config;
use std::net::{SocketAddr, UdpSocket};
use std::io;
use serde::Serialize;

const MSG_LOOPBACK          : u8 = 0x20;    // copy data
const MSG_GIMBAL            : u8 = 0x01;    // fygimbal protocol data
const MSG_FLYER_SENSORS     : u8 = 0x02;    // struct flyer_sensors
const MSG_WINCH_STATUS      : u8 = 0x03;    // struct winch_status
const MSG_WINCH_COMMAND     : u8 = 0x04;    // struct winch_command
const MSG_LEDS              : u8 = 0x05;    // apa102 data, 32 bits/pixel

pub struct BotComm {
    socket: UdpSocket,
    addrs: BotAddrs
}

impl BotComm {
    pub fn start(bus: &Bus) -> Result<BotComm, io::Error> {
        let addrs = BotAddrs::new(&bus.config.lock().unwrap());
        let socket = UdpSocket::bind(addrs.controller)?;
        start_receiver(bus.clone(), addrs.clone(), socket.try_clone()?);
        Ok(BotComm { socket, addrs })
    }

    pub fn try_clone(self: &BotComm) -> Result<BotComm, io::Error> {
        let socket = self.socket.try_clone()?;
        let addrs = self.addrs.clone();
        Ok(BotComm { socket, addrs })
    }

    fn send<T: Serialize>(self: &BotComm, addr: &SocketAddr, header: u8, body: &T) -> io::Result<()> {
        let limit = bincode::Bounded(2048);
        let packet = (header, body);
        let buf = bincode::serialize(&packet, limit).unwrap();
        self.socket.send_to(&buf[..], &addr)?;
        Ok(())
    }

    pub fn winch_command(self: &BotComm, id: usize, cmd: WinchCommand) -> io::Result<()> {
        self.send(&self.addrs.winches[id], MSG_WINCH_COMMAND, &cmd)
    }

    pub fn winch_leds<'a>(self: &'a BotComm, id: usize) -> LedWriter<'a> {
        LedWriter {
            comm: &self,
            addr: &self.addrs.winches[id]
        }
    }

    pub fn flyer_leds<'a>(self: &'a BotComm) -> LedWriter<'a> {
        LedWriter {
            comm: &self,
            addr: &self.addrs.flyer
        }
    }
}

pub struct LedWriter<'a> {
    comm: &'a BotComm,
    addr: &'a SocketAddr,
}

impl<'a> io::Write for LedWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.comm.send(self.addr, MSG_LEDS, &buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct BotAddrs {
    controller: SocketAddr,
    flyer: SocketAddr,
    winches: Vec<SocketAddr>,
}

impl BotAddrs {
    fn new(config: &Config) -> BotAddrs {
        BotAddrs {
            controller: config.controller_addr,
            flyer: config.flyer_addr,
            winches: config.winches.iter().map(|winch| { winch.addr }).collect()
        }
    }
}

fn start_receiver(bus: Bus, addrs: BotAddrs, socket: UdpSocket) {
    thread::spawn(move || {
        let mut buf = [0; 2048];
        loop {
            let (num_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
            if num_bytes >= 1 {
                handle_bot_message(&bus, &addrs, src_addr, buf[0], &buf[1..num_bytes]);
            }
        }
    });
}

fn handle_bot_message(bus: &Bus, addrs: &BotAddrs, addr: SocketAddr, code: u8, msg: &[u8]) {
    match code {

        MSG_WINCH_STATUS => {
            for (id, winch_addr) in addrs.winches.iter().enumerate() {
                if *winch_addr == addr {
                    match bincode::deserialize(msg) {
                        Err(_) => (),
                        Ok(status) => drop(bus.sender.try_send(Message::WinchStatus(id, status).timestamp())),
                    }
                }
            }
        }

        MSG_FLYER_SENSORS => {
            if addrs.flyer == addr {
                match bincode::deserialize(msg) {
                    Err(_) => (),
                    Ok(sensors) => drop(bus.sender.try_send(Message::FlyerSensors(sensors).timestamp())),
                }
            }
        }

        MSG_GIMBAL => {
            println!("Unhandled gimbal data: {:?}", msg);
        },

        MSG_LOOPBACK => (),  
        _ => (),
    }
}

