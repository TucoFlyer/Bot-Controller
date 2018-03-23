use config::WebConfig;
use iron::{Request, Response, status, Iron};
use iron::modifiers::Header;
use iron::headers::AccessControlAllowOrigin;
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

    thread::Builder::new().name("HTTP Server".into()).spawn(move || {
        let mut m = Mount::new();

        m.mount("/", web_root);

        m.mount("/ws", move |_req: &mut Request| {
            let body = serde_json::to_string(&ws_link).unwrap();
            let allow_all = Header(AccessControlAllowOrigin::Any);
            Ok(Response::with((status::Ok, body, allow_all)))
        });

        Iron::new(m).http(addr).expect("failed to start built-in HTTP server");
    }).unwrap();
}
