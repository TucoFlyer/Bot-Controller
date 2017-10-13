use message::Bus;
use qrcode::QrCode;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use open;

mod ws;
mod http;
pub mod auth;

pub fn start(bus: &Bus) {
    let web_config = bus.config.lock().unwrap().web.clone();
    let secret_key = auth::make_random_string();
    let http_uri = web_config.http_uri(&secret_key);
    let connect_string = make_connect_string(&http_uri);

    http::start(&web_config);
    ws::start(bus.clone(), &web_config, secret_key);

    store_connect_string(&connect_string, &web_config.connection_file_path).expect("can't write to connection info file");
    show_connect_string(&connect_string);
    drop(open::that(&http_uri));
}

fn make_qr_code(url: &str) -> String {
    let code = QrCode::new(url).unwrap();
    code.render::<char>().quiet_zone(true).module_dimensions(2, 1).build()
}

fn make_connect_string(url: &str) -> String {
    format!("{}\n{}\n", url, make_qr_code(url))
}

fn store_connect_string(s: &str, path: &str) -> io::Result<()> {
    let mut f = File::create(path)?;
    writeln!(f, "{}", s.replace("\n", "\r\n"))
}

fn show_connect_string(s: &str) {
    println!("\n\n\n{}\n\n", s);
}
