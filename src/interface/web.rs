//! Bot control via websockets

use bus::Bus;
use std::io::prelude::*;
use config::WebConfig;
use std::thread;
use std::fs::File;
use websocket::{Message, OwnedMessage};
use websocket::sync::{Stream, Server};
use qrcode::QrCode;
use rand::os::OsRng;
use rand::Rng;


pub fn start(bus: Bus, config: WebConfig) {
    let secret = new_secret();
    let url = http_url(&config, &secret);
    let connect_string = make_connect_string(&url);
    store_connect_string(&connect_string, &config.connection_file_path);

    start_http(&config);
    start_ws(bus, &config);

    show_connect_string(&connect_string);
}

fn start_http(config: &WebConfig) {
}

fn start_ws(bus: Bus, config: &WebConfig) {
}

fn http_url(config: &WebConfig, secret: &str) -> String {
    format!("http://{}/#{}", config.http_addr, secret)
}

fn ws_url(config: &WebConfig) -> String {
    format!("ws://{}", config.ws_addr)
}

fn new_secret() -> String {
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
    format!("{}\n\n{}", url, make_qr_code(url))
}

fn store_connect_string(s: &str, path: &str) {
    let mut f = File::create(path).expect("can't write to connection info file");
    writeln!(f, "{}\n", s);
}

fn show_connect_string(s: &str) {
    println!("\n\n\n{}\n\n", s);
}
