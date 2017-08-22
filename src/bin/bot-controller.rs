extern crate tucoflyer;
use tucoflyer::{Config, Bus, BotComm, interface, controller, watchdog};

fn main() {
    let config = Config::read_from_file("config.toml").expect("Failed to read configuration");
    let bus = Bus::new(config);
    let comm = BotComm::start(&bus).expect("Failed to start bot networking");

    interface::web::start(&bus);
    interface::gamepad::start(&bus);
    controller::start(&bus, &comm);
    watchdog::run(&bus);
}
