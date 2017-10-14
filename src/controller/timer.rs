use chrono::prelude::*;
use config::Config;
use std::time::{Duration, Instant};
use message::TICK_HZ;
use overlay::VIDEO_HZ;

pub struct ControllerTimers {
    pub tick: IntervalTimer,
    pub video_frame: IntervalTimer,    
}

impl ControllerTimers { 
    pub fn new() -> ControllerTimers {
        ControllerTimers {
            tick: IntervalTimer::new(TICK_HZ),
            video_frame: IntervalTimer::new(VIDEO_HZ),
        }
    }
}

pub struct IntervalTimer {
    period: Duration,
    timestamp: Instant,
}

impl IntervalTimer {
    pub fn new(hz: u32) -> IntervalTimer {
        IntervalTimer {
            period: Duration::new(0, 1000000000 / hz),
            timestamp: Instant::now(),
        }
    }

    pub fn poll(&mut self) -> bool {
        let now = Instant::now();
        if now > self.timestamp + self.period {
            self.timestamp = now;
            true
        } else {
            false
        }
    }
}

pub struct ConfigScheduler {
    last_poll_instant: Instant,
    last_poll_time: NaiveTime,
}

impl ConfigScheduler {
    pub fn new() -> ConfigScheduler {
        ConfigScheduler {
            last_poll_instant: Instant::now(),
            last_poll_time: Local::now().time(),
        }
    }

    pub fn poll(&mut self, config: &mut Config) -> bool {
        let timestamp = Instant::now();
        let poll_interval = Duration::new(1, 0);
        let mut changes = false;
        if timestamp > self.last_poll_instant + poll_interval {
            let last_poll_time = self.last_poll_time;
            let time = Local::now().time();
            if DailyPollInterval::new(last_poll_time, time).poll_config_changes(config) {
                changes = true;
            }
            self.last_poll_time = time;
            self.last_poll_instant = timestamp;
        }
        changes
    }
}

struct DailyPollInterval {
    from: NaiveTime,
    to: NaiveTime,
}

impl DailyPollInterval {
    fn new(from: NaiveTime, to: NaiveTime) -> DailyPollInterval {
        DailyPollInterval { from, to }
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
