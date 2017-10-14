//! Ensures the control loop is running, terminates if not.
//! Runs on the main thread.

use std::{thread, time};
use controller::ControllerPort;

pub fn run(_controller: &ControllerPort) {

	// fix me: Check for outgoing messages before declaring that we're running

	println!("Running.");

    loop {
    	// to do
        thread::sleep(time::Duration::from_millis(1000));
    }
}