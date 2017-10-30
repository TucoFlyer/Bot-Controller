//! This module is about communicating with our many robot
//! modules via a custom UDP protocol.

use message::{Message, WinchCommand};
use controller::ControllerPort;
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

#[derive(Debug)]
pub struct BotSocket {
    udp: UdpSocket,
    addrs: BotAddrs,
}

impl BotSocket {
    pub fn new(config: &Config) -> Result<BotSocket, io::Error> {
        let addrs = BotAddrs::new(config);
        let udp = UdpSocket::bind(addrs.controller)?;
        Ok(BotSocket { udp, addrs })
    }

    pub fn try_clone(&self) -> Result<BotSocket, io::Error> {
        let udp = self.udp.try_clone()?;
        let addrs = self.addrs.clone();
        Ok(BotSocket { udp, addrs })
    }

    pub fn start_receiver(&self, controller: &ControllerPort) -> GimbalPort {
        let br = BotReceiver::new(self.try_clone().unwrap(), controller);
        let gimbal_port = br.gimbal.port();
        br.start();
        gimbal_port
    }

    fn send_bytes(&self, addr: &SocketAddr, header: u8, body: &[u8]) -> io::Result<()> {
        let mut buf = vec![header];
        buf.write(body)?;
        self.udp.send_to(&buf, &addr)?;
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
            socket: &self,
        }
    }

    pub fn winch_leds<'a>(&'a self, id: usize) -> LEDWriter<'a> {
        LEDWriter {
            socket: &self,
            addr: &self.addrs.winches[id]
        }
    }

    pub fn flyer_leds<'a>(&'a self) -> LEDWriter<'a> {
        LEDWriter {
            socket: &self,
            addr: &self.addrs.flyer
        }
    }

    pub fn num_winches(&self) -> usize {
        self.addrs.winches.len()
    }
}

#[derive(Debug)]
pub struct LEDWriter<'a> {
    socket: &'a BotSocket,
    addr: &'a SocketAddr,
}

impl<'a> io::Write for LEDWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send_bytes(self.addr, MSG_LEDS, buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct GimbalWriter<'a> {
    socket: &'a BotSocket,
}

impl<'a> io::Write for GimbalWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send_bytes(&self.socket.addrs.flyer, MSG_GIMBAL, buf)?;
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
    socket: BotSocket,
    gimbal: GimbalPoller,
    controller: ControllerPort,
}

impl BotReceiver {
    fn new(socket: BotSocket, controller: &ControllerPort) -> BotReceiver {
        BotReceiver {
            socket,
            gimbal: GimbalPoller::new(),
            controller: controller.clone(),
        }
    }

    fn start(mut self) {
        thread::Builder::new().name("BotReceiver".into()).spawn(move || {
            let mut buf = [0; 2048];
            self.socket.udp.set_read_timeout(Some(GimbalPoller::read_timeout())).unwrap();
            loop {
                match self.socket.udp.recv_from(&mut buf) {
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(ref e) => panic!("Receiver thread failed: {:?}", e),

                    Ok((num_bytes, src_addr)) => {
                        if num_bytes >= 1 {
                            self.bot_message(src_addr, buf[0], &buf[1..num_bytes]);
                        }
                    }
                }
                self.gimbal.check_for_timeout(&mut self.socket.gimbal());
            }
        }).unwrap();
    }

    fn bot_message(&mut self, addr: SocketAddr, code: u8, msg: &[u8]) {
        match code {

            MSG_WINCH_STATUS => {
                let winches = &self.socket.addrs.winches;
                for (id, winch_addr) in winches.iter().enumerate() {
                    if *winch_addr == addr {
                        match bincode::deserialize(msg) {
                            Err(_) => (),
                            Ok(status) => self.controller.send(Message::WinchStatus(id, status).timestamp()),
                        }
                    }
                }
            }

            MSG_FLYER_SENSORS => {
                if self.socket.addrs.flyer == addr {
                    match bincode::deserialize(msg) {
                        Err(_) => (),
                        Ok(sensors) => self.controller.send(Message::FlyerSensors(sensors).timestamp()),
                    }
                }
            }

            MSG_GIMBAL => {
                if self.socket.addrs.flyer == addr {
                    self.gimbal.received(msg, &mut self.socket.gimbal(), &self.controller);
                }
            },

            MSG_LOOPBACK => (),  
            _ => (),
        }
    }
}

