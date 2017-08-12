//! This module is about communicating with our many robot
//! modules via a custom UDP protocol.

use bus::{Bus, Message, WinchCommand};
use std::thread;
use bincode;
use config::BotConfig;
use std::net::{SocketAddr, UdpSocket};
use std::io;

const BOT_TICK_HZ : u32 = 250;

const BOT_MSG_LOOPBACK          : u8 = 0x20;    // copy data
const BOT_MSG_GIMBAL            : u8 = 0x01;    // fygimbal protocol data
const BOT_MSG_FLYER_SENSORS     : u8 = 0x02;    // struct flyer_sensors
const BOT_MSG_WINCH_STATUS      : u8 = 0x03;    // struct winch_status
const BOT_MSG_WINCH_COMMAND     : u8 = 0x04;    // struct winch_command
const BOT_MSG_LEDS              : u8 = 0x05;    // apa102 data, 32 bits/pixel


pub struct BotCommunicator {
    socket: UdpSocket,
    config: BotConfig,
}

pub struct BotSender {
    socket: UdpSocket,
    config: BotConfig,
}

pub fn start(bus: Bus, config: BotConfig) -> Result<BotCommunicator, io::Error> {
    let socket = UdpSocket::bind(config.controller_addr)?;
    let rx_socket = socket.try_clone()?;
    start_receiver(bus, config.clone(), rx_socket);
    Ok(BotCommunicator { socket, config })
}

fn start_receiver(bus: Bus, config: BotConfig, rx_socket: UdpSocket) {
    thread::spawn(move || {
        let mut buf = [0; 2048];
        loop {
            let (num_bytes, src_addr) = rx_socket.recv_from(&mut buf).expect("Didn't receive data");
            if num_bytes >= 1 {
                handle_bot_message(&bus, &config, src_addr, buf[0], &buf[1..num_bytes]);
            }
        }
    });
}

fn handle_bot_message(bus: &Bus, config: &BotConfig, addr: SocketAddr, code: u8, msg: &[u8]) {
    match code {

        BOT_MSG_WINCH_STATUS => {
            for (id, winch) in config.winches.iter().enumerate() {
                if winch.addr == addr {
                    match bincode::deserialize(msg) {
                        Err(_) => (),
                        Ok(status) => drop(bus.sender.try_send(Message::WinchStatus(id, status))),
                    }
                }
            }
        }

        BOT_MSG_FLYER_SENSORS => {
            if config.flyer_addr == addr {
                match bincode::deserialize(msg) {
                    Err(_) => (),
                    Ok(sensors) => drop(bus.sender.try_send(Message::FlyerSensors(sensors))),
                }
            }
        }

        _ => (),
    }
}

impl BotCommunicator {
    pub fn sender(self: BotCommunicator) -> Result<BotSender, io::Error> {
        let socket = self.socket.try_clone()?;
        let config = self.config.clone();
        Ok(BotSender { socket, config })
    }
}

impl BotSender {
    pub fn winch_command(self: &BotSender, id: usize, cmd: WinchCommand) -> io::Result<()> {
        let addr = self.config.winches[id].addr;
        let limit = bincode::Bounded(2048);
        let packet = (BOT_MSG_WINCH_COMMAND, cmd);
        let buf = bincode::serialize(&packet, limit).unwrap();
        self.socket.send_to(&buf[..], &addr)?;
        Ok(())
    }
}
