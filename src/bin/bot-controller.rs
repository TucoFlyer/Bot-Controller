extern crate tucoflyer;
use tucoflyer::{SharedConfigFile, BotSocket, Controller, interface, watchdog};

fn main() {
    let config = SharedConfigFile::load("config.yaml").expect("Failed to read configuration");
    let socket = BotSocket::new(&config.get_latest()).expect("Failed to start bot networking");

    let controller = Controller::new(&config, &socket);
    let port = controller.port();
    let gimbal = socket.start_receiver(&port);
    controller.start(gimbal);

    interface::web::start(&config, &port);
    interface::gamepad::start(&config, &port);

    watchdog::run(&port);
}
