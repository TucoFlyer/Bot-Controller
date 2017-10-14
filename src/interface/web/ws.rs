use message::{Message, Command, TimestampedMessage};
use controller::ControllerPort;
use config::SharedConfigFile;
use serde_json::{to_string, from_str, Value};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use websocket;
use interface::web::auth;

// All times in milliseconds
const MIN_BATCH_PERIOD : f64 = 2.0;
const MAX_BATCH_PERIOD : f64 = 300.0;
const MAX_SEND_LATENCY : f64 = 400.0;
const PING_INTERVAL : f64 = 100.0;
const PING_TIMEOUT : f64 = 10000.0;

#[derive(Serialize, Clone, Debug)]
enum MessageToClient {
    Stream(Vec<LocalTimestampedMessage>),
    Auth(AuthChallenge),
    AuthStatus(bool),
    Error(ClientError),
}

#[derive(Deserialize, Clone, Debug)]
enum MessageToServer {
    Auth(AuthResponse),
    Command(Command),
    UpdateConfig(Value),
}

#[derive(Serialize, Clone, Debug)]
struct ClientError {
    code: ErrorCode,
    message: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
enum ErrorCode {
    ParseFailed,
    AuthRequired,
    UpdateConfigFailed,
}

pub fn start(controller: &ControllerPort, config: &SharedConfigFile, secret_key: String) {
    let controller = controller.clone();
    let config = config.clone();
    let addr = config.get_latest().web.ws_bind_addr();
    let server = websocket::sync::Server::bind(addr).expect("failed to bind to WebSocket server port");

    thread::Builder::new().name("Websocket Server".into()).spawn(move || {
        for request in server.filter_map(Result::ok) {
            let secret_key = secret_key.clone();
            let controller = controller.clone();
            let config = config.clone();
            thread::Builder::new().name("Websocket Connection".into()).spawn(move || {
                // Per-connection thread

                let (receiver, sender) = request.accept().unwrap().split().unwrap();
                let client_info = ClientInfo::new(secret_key);

                // Message sender thread, with a port for queueing new outgoing messages
                let send_port = MessageSendThread::new(client_info.clone(), sender).start();

                // Start authentication by offering the client a challenge
                send_port.direct.send(MessageToClient::Auth(client_info.challenge.clone())).unwrap();

                // Send the first config state this client will see
                send_port.stream.send(Message::ConfigIsCurrent(config.get_latest()).timestamp()).unwrap();

                // Start the message pump that reads from controller's Bus and writes to 'send_port'
                start_ws_bus_receiver(&client_info, &controller, &send_port);

                // Now handle incoming messages
                let handler = MessageHandler { client_info, send_port, controller, config };
                handler.receive(receiver);
            }).unwrap();
        }
    }).unwrap();
}

fn start_ws_bus_receiver(client_info: &ClientInfo, controller: &ControllerPort, send_port: &MessageSendPort) {
    // This thread just shuttles messages from the (fast, must not block)
    // internal message bus to the per-connection batching fifo buffer.

    let mut bus_receiver = controller.add_rx();
    let send_port = send_port.clone();
    let client_flags = client_info.flags.clone();

    thread::Builder::new().name("WebSocket bus receiver".into()).spawn(move || {
        loop {
            let msg = match bus_receiver.recv() {
                Ok(msg) => msg,
                _ => break,
            };
            if !client_flags.is_alive() {
                break;
            }
            if send_port.stream.try_send(msg).is_err() {
                // If we lost packets here, the batching flow control mechanism has failed
                // and we don't have anything better to do. Instead of silently losing data
                // let's end the connection.
                break;
            }
        }
        client_flags.kill();
    }).unwrap();
}

#[derive(Clone)]
struct ClientInfo {
    time_ref: Instant,
    challenge: AuthChallenge,
    secret_key: String,
    flags: Arc<ClientFlags>,
    flow_control: Arc<Mutex<ClientFlowControl>>,
}

impl ClientInfo {
    fn new(secret_key: String) -> ClientInfo {
        ClientInfo {
            time_ref: Instant::now(),
            challenge: AuthChallenge { challenge: auth::make_random_string() },
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
        }
    }

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

    fn authenticate(&self) {
        self.authenticated.store(true, Ordering::SeqCst);
    }

    fn is_authenticated(&self) -> bool {
        self.authenticated.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
struct ClientFlowControl {
    last_ping: f64,
    last_pong: f64,
    last_pong_latency: f64,
}

type ClientResult = Result<Option<MessageToClient>, ClientError>;

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

#[derive(Clone, Debug)]
struct MessageSendPort {
    pub stream: mpsc::SyncSender<TimestampedMessage>,
    pub direct: mpsc::SyncSender<MessageToClient>,
}

struct MessageWriter {
    client_flags: Arc<ClientFlags>,
    writer: websocket::sync::sender::Writer<TcpStream>,
}

impl MessageWriter {
    fn send(self: &mut MessageWriter, message: &MessageToClient) {
        let json = to_string(message).unwrap();
        self.send_ws_message(&websocket::OwnedMessage::Text(json));
    }

    fn ping(self: &mut MessageWriter, timestamp: f64) {
        self.send_ws_message(&websocket::OwnedMessage::Ping(timestamp.to_string().into_bytes()));
    }

    fn close(self: &mut MessageWriter) {
        self.client_flags.kill();
        self.send_ws_message(&websocket::OwnedMessage::Close(None));
    }

    fn send_ws_message(self: &mut MessageWriter, message: &websocket::OwnedMessage) {
        if self.writer.send_message(message).is_err() {
            self.client_flags.kill();
        }
    }
}

/// The message sender is the only thread that can write directly to the WebSocket.
/// It mostly batches together stream messages, but we also provide a small channel
/// for sending direct messages outside the stream.
struct MessageSendThread {
    pub port: MessageSendPort,
    writer: MessageWriter,
    client_info: ClientInfo,
    stream_reader: mpsc::Receiver<TimestampedMessage>,
    direct_reader: mpsc::Receiver<MessageToClient>,
}

impl MessageSendThread {
    fn new(client_info: ClientInfo, writer: websocket::sync::sender::Writer<TcpStream>) -> MessageSendThread {
        let (stream, stream_reader) = mpsc::sync_channel(2048);
        let (direct, direct_reader) = mpsc::sync_channel(32);
        let client_flags = client_info.flags.clone();
        MessageSendThread {
            port: MessageSendPort { stream, direct },
            writer: MessageWriter { client_flags, writer },
            client_info,
            stream_reader,
            direct_reader,
        }
    }

    fn start(mut self: MessageSendThread) -> MessageSendPort {
        let port = self.port.clone();
        thread::Builder::new().name("WebSocket MessageSendThread".into()).spawn(move || {
            // WebSocket senders iterate in batches somewhat slower than our internal control loop rate,
            // and we can skip cycles to control backlog and latency.

            let mut batch_period = MIN_BATCH_PERIOD;
            while self.client_info.flags.is_alive() {

                let now = self.client_info.relative_time(Instant::now());
                let flow_control = self.client_info.flow_control.lock().unwrap().clone();
                let need_ping = (now - flow_control.last_ping) >= PING_INTERVAL;
                let can_send = (now - flow_control.last_pong) <= MAX_SEND_LATENCY;
                let timed_out = (now - flow_control.last_pong) >= PING_TIMEOUT;

                if timed_out {
                    break;
                }

                if need_ping {
                    self.client_info.flow_control.lock().unwrap().last_ping = now;
                    self.writer.ping(now);
                }

                if can_send {
                    self.send_all_direct();
                    self.send_stream_batch();
                }

                // Base this connection's batch size on its filtered latency
                let filter_rate = 0.03;
                let filter_target = flow_control.last_pong_latency * 1.8;
                batch_period += filter_rate * (filter_target - batch_period);
                batch_period = batch_period.max(MIN_BATCH_PERIOD).min(MAX_BATCH_PERIOD);
                thread::sleep(Duration::from_millis(batch_period as u64));
            }
            self.writer.close();
        }).unwrap();
        port
    }

    fn send_all_direct(self: &mut MessageSendThread) {
        for msg in self.direct_reader.try_iter() {
            self.writer.send(&msg);
        }
    }

    fn send_stream_batch(self: &mut MessageSendThread) {
        let stream : Vec<LocalTimestampedMessage> = {
            let iter = self.stream_reader.try_iter();
            let iter = iter.filter_map(|msg| {
                if msg.timestamp < self.client_info.time_ref {
                    // Message is from before the connection was opened, can't deliver it
                    None
                } else {
                    Some(LocalTimestampedMessage::from(msg, &self.client_info))
                }
            });
            iter.collect()
        };
        if !stream.is_empty() {
            self.writer.send(&MessageToClient::Stream(stream));
        }
    }
}

struct MessageHandler {
    client_info: ClientInfo,
    send_port: MessageSendPort,
    controller: ControllerPort,
    config: SharedConfigFile,
}

impl MessageHandler {
    fn receive(&self, mut receiver: websocket::sync::receiver::Reader<TcpStream>) {
        for message in receiver.incoming_messages() {
            match message {
                Ok(m) => self.handle(m),
                Err(_) => break,
            };
            if !self.client_info.flags.is_alive() {
                break;
            }
        }
        self.client_info.flags.kill();
    }

    fn handle(&self, message: websocket::OwnedMessage) {
        match message {
            websocket::OwnedMessage::Pong(m) => self.handle_pong(m),
            websocket::OwnedMessage::Text(m) => match self.handle_json(m) {
                Err(e) => drop(self.send_port.direct.send(MessageToClient::Error(e))),
                Ok(Some(m)) => drop(self.send_port.direct.send(m)),
                Ok(None) => (),
            },
            _ => (),
        };
    }

    fn handle_pong(&self, message: Vec<u8>) {
        if let Ok(message) = String::from_utf8(message) {
            if let Ok(timestamp) = message.parse::<f64>() {
                let now = self.client_info.relative_time(Instant::now());
                if let Ok(mut flow_control) = self.client_info.flow_control.lock() {
                    if timestamp <= flow_control.last_ping + 0.01 {
                        flow_control.last_pong = timestamp;
                        flow_control.last_pong_latency = now - timestamp;
                        return;
                    }
                }
            }
        }
        // Reject unparseable timestamps, reject timestamps from the future.
        self.client_info.flags.kill();
    }

    fn handle_json(&self, message: String) -> ClientResult {
        match from_str(&message) {
            Ok(message) => self.handle_message(message),
            Err(err) => Err(ClientError {
                code: ErrorCode::ParseFailed,
                message: Some(format!("{}", err)),
            })
        }
    }

    fn handle_message(&self, message: MessageToServer) -> ClientResult {
        match message {
            MessageToServer::Auth(r) => self.try_authenticate(r),
            MessageToServer::Command(r) => self.try_command(r),
            MessageToServer::UpdateConfig(r) => self.try_update_config(r),
        }
    }

    fn try_authenticate(&self, response: AuthResponse) -> ClientResult {
        let challenge = &self.client_info.challenge.challenge;
        let key = &self.client_info.secret_key;
        let status = auth::authenticate(challenge, key, &response.digest);
        if status {
            self.client_info.flags.authenticate();
        }
        Ok(Some(MessageToClient::AuthStatus(status)))
    }

    fn try_command(&self, command: Command) -> ClientResult {
        if !self.client_info.flags.is_authenticated() {
            Err(ClientError { code: ErrorCode::AuthRequired, message: None })
        } else {
            self.controller.send(Message::Command(command).timestamp());
            Ok(None)
        }
    }

    fn try_update_config(&self, updates: Value) -> ClientResult {
        if !self.client_info.flags.is_authenticated() {
            Err(ClientError { code: ErrorCode::AuthRequired, message: None })
        } else {
            // Errors later on the command handler thread can't be reported here.
            // Do a trial run, making sure we can apply the change to a recent
            // config first.
            match self.config.get_latest().merge(updates.clone()) {
                Ok(_) => {
                    self.controller.send(Message::UpdateConfig(updates).timestamp());
                    Ok(None)
                }
                Err(e) => Err(ClientError {
                    code: ErrorCode::UpdateConfigFailed,
                    message: Some(format!("{}", e)),
                })
            }
        }
    }
}
