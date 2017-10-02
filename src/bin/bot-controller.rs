extern crate tucoflyer;
use tucoflyer::{ConfigFile, Bus, botcomm, interface, controller, watchdog};

fn main() {
    let cf = ConfigFile::load("config.yaml").expect("Failed to read configuration");
    let bus = Bus::new(cf.config.clone());
    let sender = botcomm::start(&bus).expect("Failed to start bot networking");

    interface::web::start(&bus);
    interface::gamepad::start(&bus);
    controller::start(&bus, &sender, cf);
    watchdog::run(&bus);
}
