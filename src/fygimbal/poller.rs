use std::time::Duration;
use bus::Bus;
use std::time::Instant;
use std::io;
use std::io::Write;
use std::sync::mpsc;
use fygimbal::framing::{GimbalPacket, GimbalFraming, PacketReceiver};
use bus::{GimbalCommand, GimbalStatus, Message};

#[derive(Debug, Clone)]
pub struct GimbalPort {
    packet_sender: mpsc::SyncSender<GimbalPacket>,
    cmd_sender: mpsc::SyncSender<GimbalCommand>,
}

impl GimbalPort {
    pub fn raw_packet(&self, packet: GimbalPacket) {
        self.packet_sender.send(packet);
    }

    pub fn command(&self, command: GimbalCommand) {
        self.cmd_sender.send(command);
    }
}

#[derive(Debug)]
pub struct GimbalPoller {
    receiver: PacketReceiver,
    request_timestamp: Instant,
    packet_recv: mpsc::Receiver<GimbalPacket>,
    cmd_recv: mpsc::Receiver<GimbalCommand>,
    port: GimbalPort,
}

impl GimbalPoller {

    pub fn new() -> GimbalPoller {
        let (packet_sender, packet_recv) = mpsc::sync_channel(32);
        let (cmd_sender, cmd_recv) = mpsc::sync_channel(128);
        GimbalPoller {
            receiver: PacketReceiver::new(),
            request_timestamp: Instant::now(),
            packet_recv,
            cmd_recv,
            port: GimbalPort {
                packet_sender,
                cmd_sender,
            }
        }
    }

    pub fn read_timeout() -> Duration {
        return Duration::from_millis(50);
    }

    pub fn port(&self) -> GimbalPort {
        self.port.clone()
    }

    pub fn received(&mut self, msg: &[u8], writer: &mut io::Write, bus: &Bus) {
        let mut received_anything = false;
        self.receiver.write(msg).unwrap();
        while let Some(packet) = self.receiver.next() {
            drop(bus.sender.try_send(Message::UnhandledGimbalPacket(packet).timestamp()));
            received_anything = true;
        }
        if received_anything {
            self.send_next_batch(writer);
        }
    }

    pub fn check_for_timeout(&mut self, writer: &mut io::Write) {
        if self.request_timestamp + GimbalPoller::read_timeout() < Instant::now() {
            self.send_next_batch(writer);
        }
    }

    fn send_next_batch(&mut self, writer: &mut io::Write) {
        self.request_timestamp = Instant::now();
        self.write_next_batch(&mut io::BufWriter::new(writer)).expect("Failed writing to gimbal");
    }

    fn write_next_batch(&mut self, writer: &mut io::Write) -> io::Result<()> {

        for i in 0..10 {
            for target in 0..3 {
                GimbalPacket {
                    framing: GimbalFraming::Normal,
                    command: 0x03,
                    target,
                    data: vec![0]
                }.write_to(writer)?;
            }
        }

        GimbalPacket {
            framing: GimbalFraming::Normal,
            command: 0x06,
            target: 0x00,
            data: vec![0x2C]
        }.write_to(writer)?;

        Ok(())
    }
}
