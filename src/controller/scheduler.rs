use chrono::prelude::*;
use config::Config;
use std::time::{Instant, Duration};

pub struct Scheduler {
    last_poll_instant: Instant,
    last_poll_time: NaiveTime,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            last_poll_instant: Instant::now(),
            last_poll_time: Local::now().time(),
        }
    }

    pub fn poll_config_changes(&mut self, timestamp: Instant, config: &mut Config) -> bool {
    	let poll_interval = Duration::new(1, 0);
    	let mut changes = false;
    	if timestamp > self.last_poll_instant + poll_interval {
    		let last_poll_time = self.last_poll_time;
    		let time = Local::now().time();
    		if PollInterval::new(last_poll_time, time).poll_config_changes(config) {
    			changes = true;
    		}
    		self.last_poll_time = time;
    		self.last_poll_instant = timestamp;
    	}
    	changes
    }
}

struct PollInterval {
	from: NaiveTime,
	to: NaiveTime,
}

impl PollInterval {
	fn new(from: NaiveTime, to: NaiveTime) -> PollInterval {
		PollInterval { from, to }
	}

	fn contains(&self, time: &NaiveTime) -> bool {
		if self.to < self.from {
			// Midnight wrap-around
			time > &self.from || time < &self.to
		} else {
			time > &self.from && time < &self.to
		}
	}

	fn poll_config_changes(&self, config: &mut Config) -> bool {
		let mut changes = false;
		let lighting_change = self.poll_lighting_change(config);
		if let Some(name) = lighting_change {
			if let Some(scheme) = config.lighting.saved.get(&name) {
				config.lighting.current = scheme.clone();
				changes = true;
			}
		}
		changes
	}

	fn poll_lighting_change(&self, config: &Config) -> Option<String> {
		for (time, scheme) in config.lighting.schedule.iter() {
			if self.contains(time) {
				return Some(scheme.clone());
			}
		}
		None
	}
}
