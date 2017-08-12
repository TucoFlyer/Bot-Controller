extern crate tucoflyer;
extern crate cgmath;


use cgmath::Point3;
use tucoflyer::{BotConfig, WinchConfig, WebConfig, Bus, interface, controller, watchdog, botcomm};

fn main() {
    let bus = Bus::new();

    let bot_config = BotConfig {
        controller_addr: "10.32.0.1:9024".parse().unwrap(),
        flyer_addr: "10.32.0.8:9024".parse().unwrap(),
        winches: vec![
            WinchConfig { addr: "10.32.0.10:9024".parse().unwrap(), loc: Point3::new(10.0, 10.0, 0.0) },
            WinchConfig { addr: "10.32.0.11:9024".parse().unwrap(), loc: Point3::new(10.0, -10.0, 0.0) },
            WinchConfig { addr: "10.32.0.12:9024".parse().unwrap(), loc: Point3::new(-10.0, -10.0, 0.0) },
            WinchConfig { addr: "10.32.0.13:9024".parse().unwrap(), loc: Point3::new(-10.0, 10.0, 0.0) },
        ],
    };

    let web_config = WebConfig {
        http_addr: "10.0.0.5:8080".parse().unwrap(),
        ws_addr: "10.0.0.5:8081".parse().unwrap(),
        connection_file_path: "connection.txt".to_owned(),
        web_root_path: "html".to_owned(),
    };

    interface::web::start(bus.clone(), web_config);
    interface::gamepad::start(bus.clone());
    let comm = botcomm::start(bus.clone(), bot_config.clone()).expect("Failed to start bot networking");
    controller::start(bus.clone(), comm.sender().unwrap(), bot_config);
    watchdog::run(bus);
}
