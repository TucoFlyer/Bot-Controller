extern crate tucoflyer;
use tucoflyer::{SharedConfigFile, Bus, botcomm, interface, controller, watchdog};

fn main() {
    let cf = SharedConfigFile::load("config.yaml").expect("Failed to read configuration");
    let bus = Bus::new(512);
    let sender = botcomm::start(&bus).expect("Failed to start bot networking");
    interface::web::start(&bus);
    interface::gamepad::start(&bus);
    controller::start(&bus, &sender, &cf);
    watchdog::run(&bus);
}
