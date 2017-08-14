//! Bot control via websockets

use bus::{Message, Bus};
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
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use websocket;


pub fn start(bus: Bus, config: WebConfig) {
    let secret_key = make_random_string();
    let url = http_url(&config, &secret_key);
    let connect_string = make_connect_string(&url);

    start_http(&config);
    start_ws(bus, &config, secret_key);

    store_connect_string(&connect_string, &config.connection_file_path).expect("can't write to connection info file");
    show_connect_string(&connect_string);
}

fn start_http(config: &WebConfig) {
    let addr = config.http_addr.clone();
    let mut m = Mount::new();

    m.mount("/", Static::new(&config.web_root_path));

    m.mount("/ws", {
        let url = ws_url(config);
        move |_req: &mut iron::Request| {
            Ok(iron::Response::with((iron::status::Ok, url.clone())))
        }
    });

    thread::spawn(move || {
        // Static HTTP server thread
        iron::Iron::new(m).http(addr).expect("failed to start built-in HTTP server");
    });
}

fn start_ws(bus: Bus, config: &WebConfig, secret_key: String) {
    let addr = config.ws_addr.clone();
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

fn http_url(config: &WebConfig, secret_key: &str) -> String {
    format!("http://{}/#{}", config.http_addr, secret_key)
}

fn ws_url(config: &WebConfig) -> String {
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

struct ClientState {
    alive: bool,
    authenticated: bool,
}

#[derive(Serialize)]
pub struct AuthChallenge {
    challenge: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    authenticate: String,
}

fn handle_ws_client(client: websocket::sync::Client<TcpStream>, bus: Bus, secret_key: String) {
    let challenge = AuthChallenge { challenge: make_random_string() };
    let (mut receiver, mut sender) = client.split().unwrap();
    let client_state = Arc::new(Mutex::new(ClientState {
        alive: true,
        authenticated: false,
    }));

    // Bounded queue for messages waiting to go to this client.
    // Messages can be dropped in ws_bus_receiver when this is full,
    // and they can be coalesced in ws_message_sender.
    let (rx_client_in, rx_client_out) = mpsc::sync_channel(1024);

    start_ws_bus_receiver(client_state.clone(), bus.receiver.add_stream(), rx_client_in);
    send_ws_challenge(&client_state, &challenge, &mut sender);
    start_ws_message_sender(client_state.clone(), rx_client_out, sender);
    handle_ws_message_loop(client_state, receiver, bus, &challenge);
}

fn start_ws_bus_receiver(client_state: Arc<Mutex<ClientState>>, bus_receiver: multiqueue::BroadcastReceiver<Message>, rx_client_in: mpsc::SyncSender<Message>) {
    thread::spawn(move || {
        loop {
            let msg = match bus_receiver.recv() {
                Ok(msg) => msg,
                _ => break,
            };
            if !client_state.lock().unwrap().alive {
                break;
            }

            // Drop messages if the client can't keep up
            drop(rx_client_in.try_send(msg));
        }
        client_state.lock().unwrap().alive = false;
    });
}

fn send_ws_challenge(client_state: &Arc<Mutex<ClientState>>, challenge: &AuthChallenge, sender: &mut websocket::sync::sender::Writer<TcpStream>) {
    let json = serde_json::to_string(&challenge).unwrap();
    let message = websocket::OwnedMessage::Text(json);
    if sender.send_message(&message).is_err() {
        client_state.lock().unwrap().alive = false;
    }
}

fn start_ws_message_sender(client_state: Arc<Mutex<ClientState>>, rx_client_out: mpsc::Receiver<Message>, mut sender: websocket::sync::sender::Writer<TcpStream>) {
    thread::spawn(move || {
        loop {
            // Wait for at least one message
            let mut msgvec : Vec<Message> = rx_client_out.try_iter().collect();
            if msgvec.is_empty() {
                match rx_client_out.recv() {
                    Ok(msg) => msgvec.push(msg),
                    Err(_) => break,
                }
            }
            if !client_state.lock().unwrap().alive {
                break;
            }

            let json = serde_json::to_string(&msgvec).unwrap();
            let message = websocket::OwnedMessage::Text(json);
            if sender.send_message(&message).is_err() {
                break;
            }
        }
        client_state.lock().unwrap().alive = false;
        let message = websocket::OwnedMessage::Close(None);
        drop(sender.send_message(&message));
    });
}

fn handle_ws_message_loop(client_state: Arc<Mutex<ClientState>>, mut receiver: websocket::sync::receiver::Reader<TcpStream>, bus: Bus, challenge: &AuthChallenge) {
    for message in receiver.incoming_messages() {
        let message = match message {
            Ok(m) => m,
            Err(m) => break,
        };
        println!("ws msg: {:?}", message);
    }
    client_state.lock().unwrap().alive = false;
}
