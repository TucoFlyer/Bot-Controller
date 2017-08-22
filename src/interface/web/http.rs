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

pub fn start(web_config: &WebConfig) {
    let addr = web_config.http_bind_addr();
    let ws_link = WsLink { uri: web_config.ws_uri() };
    let web_root = Static::new(&web_config.web_root_path);

    thread::spawn(move || {
        let mut m = Mount::new();

        m.mount("/", web_root);

        m.mount("/ws", move |_req: &mut iron::Request| {
            let body = serde_json::to_string(&ws_link).unwrap();
            Ok(iron::Response::with((iron::status::Ok, body)))
        });

        iron::Iron::new(m).http(addr).expect("failed to start built-in HTTP server");
    });
}
