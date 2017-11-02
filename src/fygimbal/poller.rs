use vecmath::Vector2;
use std::time::{Instant, Duration};
use std::io;
use std::mem;
use std::io::Write;
use std::sync::mpsc;
use fygimbal::framing::{GimbalPacket, GimbalFraming, PacketReceiver};
use controller::ControllerPort;
use message::Message;
use fygimbal::protocol;

const MAX_PACKETS_PER_READ_BATCH : usize = 2;
const MAX_PACKET_LENGTH : usize = 16;
const MAX_CONTINUOUS_POLL_MILLIS : u64 = 500;


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GimbalValueAddress {
    pub target: u8,
    pub index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GimbalValueData {
    pub addr: GimbalValueAddress,
    pub value: i16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GimbalValueRequest {
    pub addr: GimbalValueAddress,
    pub scope: GimbalRequestScope,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GimbalRequestScope {
    Once,
    Continuous,
}

#[derive(Debug, Clone)]
pub struct GimbalPort {
    packets: mpsc::SyncSender<GimbalPacket>,
    writes: mpsc::SyncSender<GimbalValueData>,
    requests: mpsc::SyncSender<Vec<GimbalValueRequest>>,
}

impl GimbalPort {
    pub fn send_packet(&self, packet: GimbalPacket) {
        if self.packets.try_send(packet).is_err() {
            println!("GimbalPort dropping outgoing raw packet!");
        }
    }

    pub fn set_motor_enable(&self, enable: bool) {
        for target in 0 .. protocol::NUM_AXES {
            self.send_packet(protocol::pack::motor_power(target as u8, enable as u8));
        }
    }

    pub fn write_value(&self, val: GimbalValueData) {
        if self.writes.try_send(val).is_err() {
            println!("GimbalPort dropping outgoing write!");
        }
    }

    pub fn write_control_rates(&self, rates: Vector2<i16>) {
        self.write_value(GimbalValueData {
            addr: GimbalValueAddress {
                target: protocol::target::YAW,
                index: protocol::values::CONTROL_RATE
            },
            value: rates[0]
        });
        self.write_value(GimbalValueData {
            addr: GimbalValueAddress {
                target: protocol::target::PITCH,
                index: protocol::values::CONTROL_RATE
            },
            value: rates[1]
        });
    }

    pub fn request_values(&self, reqs: Vec<GimbalValueRequest>) {
        if self.requests.try_send(reqs).is_err() {
            println!("GimbalPort dropping outgoing read requests!");
        }
    }

    pub fn request_once(&self, index: u8, target: u8) {
        self.request_values(vec![
            GimbalValueRequest {
                scope: GimbalRequestScope::Once,
                addr: GimbalValueAddress { index, target }
            }
        ]);
    }

    pub fn request_continuous(&self, index: u8, target: u8) {
        self.request_values(vec![
            GimbalValueRequest {
                scope: GimbalRequestScope::Continuous,
                addr: GimbalValueAddress { index, target }
            }
        ]);
    }
}

struct ValueTrackerItem {
    value: Option<(Instant, i16)>,
    request: Option<(Instant, GimbalRequestScope)>,
    write: Option<i16>,
}

struct ValueTracker {
    request_index: usize,
    write_index: usize,
    items: Vec<ValueTrackerItem>,
}

impl ValueTracker {
    fn new() -> ValueTracker {
        ValueTracker {
            request_index: 0,
            write_index: 0,
            items: (0 .. protocol::NUM_VALUES * protocol::NUM_AXES).map(|_| {
                ValueTrackerItem {
                    value: None,
                    request: None,
                    write: None,
                }
            }).collect()
        }
    }

    fn addr_index(addr: &GimbalValueAddress) -> Option<usize> {
        let index = addr.index as usize;
        let target = addr.target as usize;
        if index < protocol::NUM_VALUES && target < protocol::NUM_AXES {
            Some(index * protocol::NUM_AXES + target)
        } else {
            None
        }
    }

    fn index_addr(tracker_index: usize) -> GimbalValueAddress {
        GimbalValueAddress {
            index: (tracker_index / protocol::NUM_AXES) as u8,
            target: (tracker_index % protocol::NUM_AXES) as u8,
        }
    }

    fn store_value(&mut self, timestamp: Instant, data: GimbalValueData) {
        if let Some(index) = ValueTracker::addr_index(&data.addr) {
            self.items[index].value = Some((timestamp, data.value));
        }
    }

    fn store_request(&mut self, timestamp: Instant, req: GimbalValueRequest) {
        if let Some(index) = ValueTracker::addr_index(&req.addr) {
            self.items[index].request = match self.items[index].request {
                None => Some((timestamp, req.scope)),
                Some((_, GimbalRequestScope::Continuous)) => Some((timestamp, GimbalRequestScope::Continuous)),
                Some((_, GimbalRequestScope::Once)) => Some((timestamp, req.scope)),
            };
        }
    }

    fn store_write(&mut self, data: GimbalValueData) {
        if let Some(index) = ValueTracker::addr_index(&data.addr) {
            self.items[index].write = Some(data.value);
        }
    }

    fn next_write(&mut self) -> Option<GimbalValueData> {
        for _ in 0..self.items.len() {
            let index = self.write_index;
            self.write_index = (self.write_index + 1) % self.items.len();

            if let Some(value) = mem::replace(&mut self.items[index].write, None) {
                return Some(GimbalValueData {
                    addr: ValueTracker::index_addr(index),
                    value,
                })
            }
        }
        None
    }

    fn next_request(&mut self) -> Option<GimbalValueAddress> {
        for _ in 0..self.items.len() {
            let index = self.request_index;
            self.request_index = (self.request_index + 1) % self.items.len();
            match self.items[index].request {

                None => (),

                Some((_, GimbalRequestScope::Once)) => {
                    self.items[index].request = None;
                    return Some(ValueTracker::index_addr(index));
                }

                Some((ts, GimbalRequestScope::Continuous)) => {
                    if Instant::now() > ts + Duration::from_millis(MAX_CONTINUOUS_POLL_MILLIS) {
                        self.items[index].request = None;                    
                    } else {
                        return Some(ValueTracker::index_addr(index));
                    }
                }
            }
        }
        None
    }
}

pub struct GimbalPoller {
    receiver: PacketReceiver,
    batch_timestamp: Instant,
    pending_read: Option<GimbalValueAddress>,
    value_tracker: ValueTracker,

    port: GimbalPort,
    packets: mpsc::Receiver<GimbalPacket>,
    writes: mpsc::Receiver<GimbalValueData>,
    requests: mpsc::Receiver<Vec<GimbalValueRequest>>,
}

impl GimbalPoller {
    pub fn new() -> GimbalPoller {
        let packets = mpsc::sync_channel(256);
        let writes = mpsc::sync_channel(256);
        let requests = mpsc::sync_channel(256);

        GimbalPoller {
            receiver: PacketReceiver::new(),
            batch_timestamp: Instant::now(),
            pending_read: None,
            value_tracker: ValueTracker::new(),

            port: GimbalPort {
                packets: packets.0,
                writes: writes.0,
                requests: requests.0,
            },
            packets: packets.1,
            writes: writes.1,
            requests: requests.1,
        }
    }

    pub fn read_timeout() -> Duration {
        return Duration::from_millis(50);
    }

    pub fn port(&self) -> GimbalPort {
        self.port.clone()
    }

    pub fn received(&mut self, msg: &[u8], writer: &mut io::Write, controller: &ControllerPort) {
        let mut received_anything = false;
        self.receiver.write(msg).unwrap();
        while let Some(packet) = self.receiver.next() {
            self.handle_packet(packet, controller);
            received_anything = true;
        }
        if received_anything {
            self.send_next_batch(writer);
        }
    }

    fn handle_packet(&mut self, packet: GimbalPacket, controller: &ControllerPort) {
        if packet.framing == GimbalFraming::Normal {
            if packet.target == protocol::target::HOST {
                if packet.command == protocol::cmd::GET_VALUE {
                    if let Some(addr) = mem::replace(&mut self.pending_read, None) {
                        if let Ok(value) = protocol::unpack::get_value(&packet) {

                            let data = GimbalValueData { addr, value };
                            let tsm = Message::GimbalValue(data.clone()).timestamp();
    
                            self.value_tracker.store_value(tsm.timestamp, data);
                            controller.send(tsm);
                            return;
                        }
                    }
                }
            }
        }
        controller.send(Message::UnhandledGimbalPacket(packet).timestamp());
    }

    pub fn check_for_timeout(&mut self, writer: &mut io::Write) {
        if self.batch_timestamp + GimbalPoller::read_timeout() < Instant::now() {
            self.send_next_batch(writer);
        }
    }

    fn send_next_batch(&mut self, writer: &mut io::Write) {
        self.drain_command_queues();
        self.batch_timestamp = Instant::now();
        self.write_next_batch(&mut io::BufWriter::new(writer)).expect("Failed writing to gimbal");
    }

    fn drain_command_queues(&mut self) {
        for data in self.writes.try_iter() {
            self.value_tracker.store_write(data);
        }
        for v in self.requests.try_iter() {
            let now = Instant::now();
            for req in v {
                self.value_tracker.store_request(now, req);
            }
        }
    }

    fn write_next_batch(&mut self, writer: &mut io::Write) -> io::Result<()> {
        let mut remaining = MAX_PACKETS_PER_READ_BATCH;

        // Spend all packets except the last one on a mix of writes
        while remaining > 1 {
            if let Ok(packet) = self.packets.try_recv() {
                if packet.data.len() <= MAX_PACKET_LENGTH {
                    packet.write_to(writer)?;
                    remaining -= 1;
                }
            }
            else if let Some(data) = self.value_tracker.next_write() {
                let packet = protocol::pack::set_value(data.addr.target, data.addr.index, data.value);
                packet.write_to(writer)?;
                remaining -= 1;
            }
            else {
                break;
            }
        }

        // The last packet should be a read, so we can wait for the response
        assert!(remaining >= 1);
        if let Some(addr) = self.value_tracker.next_request() {
            let packet = protocol::pack::get_value(addr.target, addr.index);
            self.pending_read = Some(addr);
            packet.write_to(writer)?;
        }
        Ok(())
    }
}
