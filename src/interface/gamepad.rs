//! Bot control via a local gamepad

use bus::{Bus, Command, Message, ControllerMode, ManualControlAxis};
use gilrs::{Event, Button, Axis, Gilrs};
use std::thread;
use std::time::Duration;


pub fn start( bus: Bus ) {
    thread::spawn(move || {
        let mut gil = Gilrs::new();
        loop {
            for (_id, event) in gil.poll_events() {

                let cmd = match event {
                    Event::Connected => Some(Command::ManualControlReset),
                    Event::Disconnected => Some(Command::ManualControlReset),
                    Event::ButtonPressed(Button::East, _) => Some(Command::SetMode(ControllerMode::Halted)),
                    Event::ButtonPressed(Button::South, _) => Some(Command::SetMode(ControllerMode::Manual)),
                    Event::AxisChanged(Axis::RightStickX, v, _) => Some(Command::ManualControlValue(ManualControlAxis::CameraYaw, v)),
                    Event::AxisChanged(Axis::RightStickY, v, _) => Some(Command::ManualControlValue(ManualControlAxis::CameraPitch, v)),
                    Event::AxisChanged(Axis::LeftStickX, v, _) => Some(Command::ManualControlValue(ManualControlAxis::RelativeX, v)),
                    Event::AxisChanged(Axis::LeftStickY, v, _) => Some(Command::ManualControlValue(ManualControlAxis::RelativeY, v)),
                    Event::AxisChanged(Axis::LeftTrigger2, v, _) => Some(Command::ManualControlValue(ManualControlAxis::RelativeZ, -v)),
                    Event::AxisChanged(Axis::RightTrigger2, v, _) => Some(Command::ManualControlValue(ManualControlAxis::RelativeZ, v)),
                    _ => None,
                };

                match cmd {
                    Some(c) => drop(bus.sender.try_send(Message::Command(c))),
                    None => (),
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });
}
