use bus::Bus;
use config::WebConfig;
use qrcode::QrCode;
use rand::os::OsRng;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
use std::io;

mod ws;
mod http;

pub fn start(bus: Bus, config: WebConfig) {
    let secret_key = make_random_string();
    let connect_string = make_connect_string(&config.http_uri(&secret_key));

    http::start(&config);
    ws::start(bus, &config, secret_key);

    store_connect_string(&connect_string, &config.connection_file_path).expect("can't write to connection info file");
    show_connect_string(&connect_string);
}

pub fn make_random_string() -> String {
    let mut rng = OsRng::new().expect("can't access the OS random number generator");
    rng.gen_ascii_chars().take(30).collect()
}

fn make_qr_code(url: &str) -> String {
    let code = QrCode::new(url).unwrap();
    code.render::<char>().quiet_zone(false).module_dimensions(2, 1).build()
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
