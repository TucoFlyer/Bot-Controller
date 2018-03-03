use controller::ControllerPort;
use qrcode::QrCode;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use config::SharedConfigFile;
use open;

mod ws;
mod http;
pub mod auth;

pub fn start(config: &SharedConfigFile, controller: &ControllerPort) {
    let web_config = config.get_latest().web;
    let secret_key = auth::make_random_string();
    let connect_string = make_connect_string(&web_config.http_uri(&secret_key, 0));

    http::start(&web_config);
    ws::start(controller, config, secret_key.clone());

    store_connect_string(&connect_string, &web_config.connection_file_path).expect("can't write to connection info file");
    show_connect_string(&connect_string);

    if web_config.open_browser {
        drop(open::that(&web_config.http_uri(&secret_key, web_config.browser_port_override)));
    }
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
