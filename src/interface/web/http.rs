use config::WebConfig;
use iron;
use mount::Mount;
use staticfile::Static;
use std::thread;
use serde_json;

#[derive(Serialize)]
struct WsLink {
	uri: String
}

pub fn start(config: &WebConfig) {
    let addr = config.http_bind_addr();
    let ws_link = WsLink { uri: config.ws_uri() };
    let web_root = Static::new(&config.web_root_path);

    thread::spawn(move || {
        let mut m = Mount::new();

        m.mount("/", web_root);

        m.mount("/ws", move |_req: &mut iron::Request| {
            let body = serde_json::to_string(&ws_link).unwrap();
            Ok(iron::Response::with((iron::status::Ok, body)))
        });

        // Static HTTP server thread
        iron::Iron::new(m).http(addr).expect("failed to start built-in HTTP server");
    });
}
