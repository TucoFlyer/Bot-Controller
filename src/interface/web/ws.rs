use bus::{Message, TimestampedMessage, Bus};
use config::WebConfig;
use multiqueue;
use serde_json;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use websocket;
use interface::web::make_random_string;

pub fn start(bus: Bus, config: &WebConfig, secret_key: String) {
    let addr = config.ws_bind_addr();
    thread::spawn(move || {
        // Websocket acceptor thread
        let server = websocket::sync::Server::bind(addr).expect("failed to bind to WebSocket server port");
        for request in server.filter_map(Result::ok) {
            let secret_key = secret_key.clone();
            let bus = bus.clone();
            thread::spawn(move || {
                // Per-connection thread
                handle_ws_client(request.accept().unwrap(), bus, secret_key);
            });
        }
    });
}

// All times in milliseconds
const MIN_BATCH_PERIOD : f64 = 2.0;
const MAX_BATCH_PERIOD : f64 = 300.0;
const MAX_SEND_LATENCY : f64 = 400.0;
const PING_INTERVAL : f64 = 100.0;
const PING_TIMEOUT : f64 = 10000.0;

#[derive(Clone)]
struct ClientInfo {
    time_ref: Instant,
    challenge: AuthChallenge,
    secret_key: String,
    flags: Arc<ClientFlags>,
    flow_control: Arc<Mutex<ClientFlowControl>>,
}

impl ClientInfo {
    fn relative_time(&self, instant: Instant) -> f64 {
        // Use millisecond floats on the websocket interface, it's convenient for Javascript
        let duration = instant.duration_since(self.time_ref);
        duration.as_secs() as f64 * 1e3 + duration.subsec_nanos() as f64 * 1e-6
    }
}

struct ClientFlags {
    alive: AtomicBool,
    authenticated: AtomicBool,
}

impl ClientFlags {
    fn kill(&self) {
        self.alive.store(false, Ordering::SeqCst);
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
struct ClientFlowControl {
    last_ping: f64,
    last_pong: f64,
    last_pong_latency: f64,
}

#[derive(Serialize, Clone, Debug)]
enum MessageToClient {
    Auth(AuthChallenge),
    Stream(Vec<LocalTimestampedMessage>),
}

#[derive(Deserialize, Clone, Debug)]
enum MessageToServer {
    Auth(AuthResponse),
}

#[derive(Serialize, Clone, Debug)]
struct AuthChallenge {
    pub challenge: String,
}

#[derive(Deserialize, Clone, Debug)]
struct AuthResponse {
    pub digest: String,
}

#[derive(Serialize, Clone, Debug)]
struct LocalTimestampedMessage {
    pub timestamp: f64,
    pub message: Message,
}

impl LocalTimestampedMessage {
    fn from(tsm: TimestampedMessage, client_info: &ClientInfo) -> LocalTimestampedMessage {
        LocalTimestampedMessage {
            timestamp: client_info.relative_time(tsm.timestamp),
            message: tsm.message,
        }
    }
}

struct MessageSender {
    client_flags: Arc<ClientFlags>,
    sender: websocket::sync::sender::Writer<TcpStream>,
}

impl MessageSender {
    fn send(self: &mut MessageSender, message: &MessageToClient) {
        let json = serde_json::to_string(message).unwrap();
        self.send_ws_message(&websocket::OwnedMessage::Text(json));
    }

    fn ping(self: &mut MessageSender, timestamp: f64) {
        self.send_ws_message(&websocket::OwnedMessage::Ping(timestamp.to_string().into_bytes()));
    }

    fn close(self: &mut MessageSender) {
        self.client_flags.kill();
        self.send_ws_message(&websocket::OwnedMessage::Close(None));
    }

    fn send_ws_message(self: &mut MessageSender, message: &websocket::OwnedMessage) {
        if self.sender.send_message(message).is_err() {
            self.client_flags.kill();
        }
    }
}

fn handle_ws_client(client: websocket::sync::Client<TcpStream>, bus: Bus, secret_key: String) {
    let (receiver, sender) = client.split().unwrap();

    let client_info = ClientInfo {
        time_ref: Instant::now(),
        challenge: AuthChallenge { challenge: make_random_string() },
        secret_key,
        flags: Arc::new(ClientFlags {
            alive: AtomicBool::new(true),
            authenticated: AtomicBool::new(false),
        }),
        flow_control: Arc::new(Mutex::new(ClientFlowControl {
            last_ping: -PING_INTERVAL,
            last_pong: 0.0,
            last_pong_latency: 0.0,
        })),
    };

    let mut sender = MessageSender {
        client_flags: client_info.flags.clone(),
        sender
    };

    sender.send(&MessageToClient::Auth(client_info.challenge.clone()));

    // Bounded queue for messages waiting to go to this client.
    // Messages can be dropped in ws_bus_receiver when this is full,
    // and they can be coalesced in ws_message_sender.
    let (rx_client_in, rx_client_out) = mpsc::sync_channel(2048);

    start_ws_bus_receiver(client_info.clone(), bus.receiver.add_stream(), rx_client_in);
    start_ws_message_sender(client_info.clone(), rx_client_out, sender);
    handle_ws_message_loop(client_info, receiver, bus);
}

fn start_ws_bus_receiver(client_info: ClientInfo, bus_receiver: multiqueue::BroadcastReceiver<TimestampedMessage>, rx_client_in: mpsc::SyncSender<TimestampedMessage>) {
    thread::spawn(move || {
        loop {
            let msg = match bus_receiver.recv() {
                Ok(msg) => msg,
                _ => break,
            };
            if !client_info.flags.is_alive() {
                break;
            }
            if rx_client_in.try_send(msg).is_err() {
                // If we lost packets here, the batching flow control mechanism has failed
                // and we don't have anything better to do. Instead of silently losing data
                // let's end the connection.
                break;
            }
        }
        client_info.flags.kill();
    });
}

fn start_ws_message_sender(client_info: ClientInfo, rx_client_out: mpsc::Receiver<TimestampedMessage>, mut sender: MessageSender) {
    thread::spawn(move || {
        let convert_msg = |msg| { LocalTimestampedMessage::from(msg, &client_info) };

        // WebSocket senders iterate in batches somewhat slower than our internal control loop rate,
        // and we can skip cycles to control backlog and latency.

        let mut batch_period = MIN_BATCH_PERIOD;
        while client_info.flags.is_alive() {

            let now = client_info.relative_time(Instant::now());
            let flow_control = client_info.flow_control.lock().unwrap().clone();
            let need_ping = (now - flow_control.last_ping) >= PING_INTERVAL;
            let can_send = (now - flow_control.last_pong) <= MAX_SEND_LATENCY;
            let timed_out = (now - flow_control.last_pong) >= PING_TIMEOUT;

            if timed_out {
                break;
            }

            if need_ping {
                client_info.flow_control.lock().unwrap().last_ping = now;
                sender.ping(now);
            }

            if can_send {
                let msgvec : Vec<LocalTimestampedMessage> = rx_client_out.try_iter().map(&convert_msg).collect();
                if !msgvec.is_empty() {
                    sender.send(&MessageToClient::Stream(msgvec));
                }
            }

            // Base this connection's batch size on its filtered latency
            let filter_rate = 0.03;
            let filter_target = flow_control.last_pong_latency * 1.8;
            batch_period += filter_rate * (filter_target - batch_period);
            batch_period = batch_period.max(MIN_BATCH_PERIOD).min(MAX_BATCH_PERIOD);
            thread::sleep(Duration::from_millis(batch_period as u64));
        }
        sender.close();
    });
}

fn handle_ws_message_loop(client_info: ClientInfo, mut receiver: websocket::sync::receiver::Reader<TcpStream>, bus: Bus) {
    for message in receiver.incoming_messages() {
        let message = match message {
            Ok(m) => m,
            Err(_) => break,
        };
        match message {

            websocket::OwnedMessage::Pong(m) => handle_ws_pong(&client_info, m),
            websocket::OwnedMessage::Text(m) => handle_ws_text(&client_info, &bus, m),

            _ => (),
        };
        if !client_info.flags.is_alive() {
            break;
        }
    }
    client_info.flags.kill();
}

fn handle_ws_pong(client_info: &ClientInfo, message: Vec<u8>) {
    if let Ok(message) = String::from_utf8(message) {
        if let Ok(timestamp) = message.parse::<f64>() {
            let now = client_info.relative_time(Instant::now());
            if let Ok(mut flow_control) = client_info.flow_control.lock() {
                if timestamp <= flow_control.last_ping + 0.01 {
                    flow_control.last_pong = timestamp;
                    flow_control.last_pong_latency = now - timestamp;
                    return;
                }
            }
        }
    }
    // Reject unparseable timestamps, reject timestamps from the future.
    client_info.flags.kill();
}

fn handle_ws_text(_client_info: &ClientInfo, _bus: &Bus, message: String) {
    println!("ws msg text {:?}", message);
}
