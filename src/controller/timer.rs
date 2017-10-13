/// Timekeeping for periodic events that occur less often than once per message

use std::time::{Duration, Instant};

struct IntervalTimer {
    period: Duration,
    timestamp: Instant,
}

impl IntervalTimer {
    fn new(hz: u32) -> IntervalTimer {
        IntervalTimer {
            period: Duration::new(0, 1000000000 / hz),
            timestamp: Instant::now(),
        }
    }

    fn poll(&mut self) -> bool {
        let now = Instant::now();
        if now > self.timestamp + self.period {
            self.timestamp = now;
            true
        } else {
            false
        }
    }
}
