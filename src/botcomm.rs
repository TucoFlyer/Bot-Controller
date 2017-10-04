//! This module is about communicating with our many robot
//! modules via a custom UDP protocol.

use bus::{Bus, Message, WinchCommand};
use std::thread;
use bincode;
use config::Config;
use std::net::{SocketAddr, UdpSocket};
use std::io;
use std::io::Write;
use serde::Serialize;
use fygimbal::{GimbalPoller, GimbalPort};

const MSG_LOOPBACK          : u8 = 0x20;    // copy data
const MSG_GIMBAL            : u8 = 0x01;    // fygimbal protocol data
const MSG_FLYER_SENSORS     : u8 = 0x02;    // struct flyer_sensors
const MSG_WINCH_STATUS      : u8 = 0x03;    // struct winch_status
const MSG_WINCH_COMMAND     : u8 = 0x04;    // struct winch_command
const MSG_LEDS              : u8 = 0x05;    // apa102 data, 32 bits/pixel

pub fn start(bus: &Bus) -> Result<BotSender, io::Error> {
    let addrs = BotAddrs::new(&bus.config.lock().unwrap());
    let socket = UdpSocket::bind(addrs.controller)?;

    let recv = BotReceiver {
        bus: bus.clone(),
        addrs: addrs.clone(),
        socket: socket.try_clone()?,
        gimbal: GimbalPoller::new(),
    };

    let sender = BotSender {
        socket,
        addrs,
        gimbal: recv.gimbal.port(),
    };

    recv.start(sender.try_clone()?);
    Ok(sender)
}

#[derive(Debug)]
pub struct BotSender {
    socket: UdpSocket,
    addrs: BotAddrs,
    gimbal: GimbalPort,
}

impl BotSender {
    pub fn try_clone(&self) -> Result<BotSender, io::Error> {
        let socket = self.socket.try_clone()?;
        let addrs = self.addrs.clone();
        let gimbal = self.gimbal.clone();
        Ok(BotSender { socket, addrs, gimbal })
    }

    fn send_bytes(&self, addr: &SocketAddr, header: u8, body: &[u8]) -> io::Result<()> {
        let mut buf = vec![header];
        buf.write(body)?;
        self.socket.send_to(&buf, &addr)?;
        Ok(())
    }

    fn send<T: Serialize>(&self, addr: &SocketAddr, header: u8, body: &T) -> io::Result<()> {
        let limit = bincode::Bounded(2048);
        let bytes = bincode::serialize(body, limit).unwrap();
        self.send_bytes(addr, header, &bytes)
    }

    pub fn winch_command(&self, id: usize, cmd: WinchCommand) -> io::Result<()> {
        self.send(&self.addrs.winches[id], MSG_WINCH_COMMAND, &cmd)
    }

    fn gimbal<'a>(&'a self) -> GimbalWriter<'a> {
        GimbalWriter {
            sender: &self,
        }
    }

    pub fn winch_leds<'a>(&'a self, id: usize) -> LEDWriter<'a> {
        LEDWriter {
            sender: &self,
            addr: &self.addrs.winches[id]
        }
    }

    pub fn flyer_leds<'a>(&'a self) -> LEDWriter<'a> {
        LEDWriter {
            sender: &self,
            addr: &self.addrs.flyer
        }
    }

    pub fn num_winches(&self) -> usize {
        self.addrs.winches.len()
    }
}

#[derive(Debug)]
pub struct LEDWriter<'a> {
    sender: &'a BotSender,
    addr: &'a SocketAddr,
}

impl<'a> io::Write for LEDWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sender.send_bytes(self.addr, MSG_LEDS, buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct GimbalWriter<'a> {
    sender: &'a BotSender,
}

impl<'a> io::Write for GimbalWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sender.send_bytes(&self.sender.addrs.flyer, MSG_GIMBAL, buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
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

struct BotReceiver {
    bus: Bus,
    addrs: BotAddrs,
    socket: UdpSocket,
    gimbal: GimbalPoller,
}

impl BotReceiver {

    fn start(mut self, sender: BotSender) {
        thread::spawn(move || {
            let mut buf = [0; 2048];
            self.socket.set_read_timeout(Some(GimbalPoller::read_timeout())).unwrap();
            loop {
                match self.socket.recv_from(&mut buf) {
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(ref e) => panic!("Receiver thread failed: {:?}", e),

                    Ok((num_bytes, src_addr)) => {
                        if num_bytes >= 1 {
                            self.bot_message(&sender, src_addr, buf[0], &buf[1..num_bytes]);
                        }
                    }
                }
                self.gimbal.check_for_timeout(&mut sender.gimbal());
            }
        });
    }

    fn bot_message(&mut self, sender: &BotSender, addr: SocketAddr, code: u8, msg: &[u8]) {
        match code {

            MSG_WINCH_STATUS => {
                for (id, winch_addr) in self.addrs.winches.iter().enumerate() {
                    if *winch_addr == addr {
                        match bincode::deserialize(msg) {
                            Err(_) => (),
                            Ok(status) => drop(self.bus.sender.try_send(Message::WinchStatus(id, status).timestamp())),
                        }
                    }
                }
            }

            MSG_FLYER_SENSORS => {
                if self.addrs.flyer == addr {
                    match bincode::deserialize(msg) {
                        Err(_) => (),
                        Ok(sensors) => drop(self.bus.sender.try_send(Message::FlyerSensors(sensors).timestamp())),
                    }
                }
            }

            MSG_GIMBAL => {
                if self.addrs.flyer == addr {
                    self.gimbal.received(msg, &mut sender.gimbal(), &self.bus);
                }
            },

            MSG_LOOPBACK => (),  
            _ => (),
        }
    }
}

