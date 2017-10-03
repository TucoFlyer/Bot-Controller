use std::time::Duration;
use bus::Bus;
use botcomm::BotSender;

#[derive(Debug, Clone)]
pub struct GimbalPort {

}


#[derive(Debug)]
pub struct GimbalPoller {

}

impl GimbalPoller {

	pub fn new() -> GimbalPoller {
		GimbalPoller {

		}
	}

	pub fn read_timeout() -> Duration {
		return Duration::from_millis(50);
	}

	pub fn port(&self) -> GimbalPort {
		GimbalPort {

		}
	}

    pub fn received(&mut self, msg: &[u8], sender: &BotSender, bus: &Bus) {

    }

    pub fn check_for_timeout(&mut self, sender: &BotSender) {

    }

}
