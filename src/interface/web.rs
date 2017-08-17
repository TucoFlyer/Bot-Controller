//! Bot control via websockets

use bus::{Message, TimestampedMessage, Bus};
use config::WebConfig;
use iron;
use mount::Mount;
use multiqueue;
use qrcode::QrCode;
use rand::os::OsRng;
use rand::Rng;
use serde_json;
use staticfile::Static;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::net::{TcpStream, IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use websocket;

pub fn start(bus: Bus, config: WebConfig) {
    let secret_key = make_random_string();
    let url = http_uri(&config, &secret_key);
    let connect_string = make_connect_string(&url);

    start_http(&config);
    start_ws(bus, &config, secret_key);

    store_connect_string(&connect_string, &config.connection_file_path).expect("can't write to connection info file");
    show_connect_string(&connect_string);
}

fn all_if_addr() -> IpAddr {
    // Bind to all interfaces; we need at least localhost and the LAN
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

fn start_http(config: &WebConfig) {
    let addr = SocketAddr::new(all_if_addr(), config.http_addr.port());
    let mut m = Mount::new();

    m.mount("/", Static::new(&config.web_root_path));

    m.mount("/ws", {
        let body = json!({ "uri": ws_uri(config) });
        move |_req: &mut iron::Request| {
            let body = serde_json::to_string(&body).unwrap();
            Ok(iron::Response::with((iron::status::Ok, body)))
        }
    });

    thread::spawn(move || {
        // Static HTTP server thread
        iron::Iron::new(m).http(addr).expect("failed to start built-in HTTP server");
    });
}

fn start_ws(bus: Bus, config: &WebConfig, secret_key: String) {
    let addr = SocketAddr::new(all_if_addr(), config.ws_addr.port());
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

fn http_uri(config: &WebConfig, secret_key: &str) -> String {
    format!("http://{}/#?k={}", config.http_addr, secret_key)
}

fn ws_uri(config: &WebConfig) -> String {
    format!("ws://{}", config.ws_addr)
}

fn make_random_string() -> String {
    let mut rng = OsRng::new().expect("can't access the OS random number generator");
    rng.gen_ascii_chars().take(30).collect()
}

fn make_qr_code(url: &str) -> String {
    let code = QrCode::new(url).unwrap();
    code.render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build()
}

fn make_connect_string(url: &str) -> String {
    format!("{}\n\n{}\n", url, make_qr_code(url))
}

fn store_connect_string(s: &str, path: &str) -> io::Result<()> {
    let mut f = File::create(path)?;
    writeln!(f, "{}", s.replace("\n", "\r\n"))
}

fn show_connect_string(s: &str) {
    println!("\n\n\n{}\n\n", s);
}

struct ClientFlags {
    alive: AtomicBool,
    authenticated: AtomicBool,
}

// All times in milliseconds
const SEND_BATCH_PERIOD : u64 = 10;
const MAX_SEND_LATENCY : f64 = 400.0;
const PING_INTERVAL : f64 = 100.0;
const PING_TIMEOUT : f64 = 10000.0;

#[derive(Debug, Clone)]
struct ClientFlowControl {
    last_ping: f64,
    last_pong: f64,
}

#[derive(Clone)]
struct ClientInfo {
    time_ref: Instant,
    challenge: AuthChallenge,
    flags: Arc<ClientFlags>,
    flow_control: Arc<Mutex<ClientFlowControl>>,
}

impl ClientInfo {
    fn relative_time(&self, instant: Instant) -> f64 {
        // Use millisecond floats on the websocket interface, it's convenient for Javascript
        let duration = instant.duration_since(self.time_ref);
        duration.as_secs() as f64 * 1e3 + duration.subsec_nanos() as f64 * 1e-6
    }

    fn kill(&self) {
        self.flags.alive.store(false, Ordering::SeqCst);
    }

    fn is_alive(&self) -> bool {
        self.flags.alive.load(Ordering::SeqCst)
    }
}

#[derive(Serialize, Clone, Debug)]
struct AuthChallenge {
    pub challenge: String,
}

#[derive(Deserialize, Debug)]
struct AuthResponse {
    pub authenticate: String,
}

#[derive(Serialize, Debug)]
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

fn handle_ws_client(client: websocket::sync::Client<TcpStream>, bus: Bus, secret_key: String) {
    let (mut receiver, mut sender) = client.split().unwrap();

    let client_info = ClientInfo {
        time_ref: Instant::now(),
        challenge: AuthChallenge { challenge: make_random_string() },
        flags: Arc::new(ClientFlags {
            alive: AtomicBool::new(true),
            authenticated: AtomicBool::new(false),
        }),
        flow_control: Arc::new(Mutex::new(ClientFlowControl {
            last_ping: -PING_INTERVAL,
            last_pong: 0.0,
        })),
    };

    // Bounded queue for messages waiting to go to this client.
    // Messages can be dropped in ws_bus_receiver when this is full,
    // and they can be coalesced in ws_message_sender.
    let (rx_client_in, rx_client_out) = mpsc::sync_channel(1024);

    start_ws_bus_receiver(client_info.clone(), bus.receiver.add_stream(), rx_client_in);
    send_ws_challenge(&client_info, &mut sender);
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
            if !client_info.is_alive() {
                break;
            }

            // Drop messages if the client can't keep up
            drop(rx_client_in.try_send(msg));
        }
        client_info.kill();
    });
}

fn send_ws_challenge(client_info: &ClientInfo, sender: &mut websocket::sync::sender::Writer<TcpStream>) {
    let json = serde_json::to_string(&client_info.challenge).unwrap();
    let message = websocket::OwnedMessage::Text(json);
    if sender.send_message(&message).is_err() {
        client_info.kill();
    }
}

fn start_ws_message_sender(client_info: ClientInfo, rx_client_out: mpsc::Receiver<TimestampedMessage>, mut sender: websocket::sync::sender::Writer<TcpStream>) {
    thread::spawn(move || {
        let convertMsg = |msg| { LocalTimestampedMessage::from(msg, &client_info) };
        let mut send = |ws_msg| {
            if sender.send_message(&ws_msg).is_err() {
                client_info.kill();
            }
        };

        // WebSocket senders iterate in batches somewhat slower than our internal control loop rate,
        // and we can skip cycles to control backlog and latency.

        while client_info.is_alive() {

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
                send(websocket::OwnedMessage::Ping(now.to_string().into_bytes()));
            }

            if can_send {
                let mut msgvec : Vec<LocalTimestampedMessage> = rx_client_out.try_iter().map(&convertMsg).collect();
                if !msgvec.is_empty() {
                    let json = serde_json::to_string(&msgvec).unwrap();
                    send(websocket::OwnedMessage::Text(json));    
                }
            }

            thread::sleep(Duration::from_millis(SEND_BATCH_PERIOD));
        }

        client_info.kill();
        send(websocket::OwnedMessage::Close(None)); 
    });
}

fn handle_ws_message_loop(client_info: ClientInfo, mut receiver: websocket::sync::receiver::Reader<TcpStream>, bus: Bus) {
    for message in receiver.incoming_messages() {
        let message = match message {
            Ok(m) => m,
            Err(m) => break,
        };
        match message {

            websocket::OwnedMessage::Pong(m) => handle_ws_pong(&client_info, &bus, m),
            websocket::OwnedMessage::Text(m) => handle_ws_text(&client_info, &bus, m),

            _ => (),
        };
        if !client_info.is_alive() {
            break;
        }
    }
    client_info.kill();
}

fn handle_ws_pong(client_info: &ClientInfo, bus: &Bus, message: Vec<u8>) {
    if let Ok(message) = String::from_utf8(message) {
        if let Ok(timestamp) = message.parse::<f64>() {
            if let Ok(mut flow_control) = client_info.flow_control.lock() {
                if timestamp <= flow_control.last_ping + 0.01 {
                    flow_control.last_pong = timestamp;
                    return;
                }
            }
        }
    }
    // Reject unparseable timestamps, reject timestamps from the future.
    client_info.kill();
}

fn handle_ws_text(client_info: &ClientInfo, bus: &Bus, message: String) {
    println!("ws msg text {:?}", message);
}
