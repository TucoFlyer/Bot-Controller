//! Ensures the control loop is running, terminates if not.
//! Runs on the main thread.

use std::{thread, time};
use message::Bus;

pub fn run(_bus: &Bus) {

	// fix me: Check for outgoing messages before declaring that we're running

	println!("Running.");

    loop {
    	// to do
        thread::sleep(time::Duration::from_millis(1000));
    }
}