//! Bot control via a local gamepad

use bus::{Bus, Command, Message, ControllerMode, ManualControlAxis};
use gilrs::{Event, Button, Axis, Gilrs};
use std::thread;
use std::time::Duration;


struct State {
    left_z: f32,
    right_z: f32,
}

impl State {
    fn new() -> State {
        State {
            left_z: 0.0,
            right_z: 0.0
        }
    }

    fn z_command(self: &State) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeZ, self.right_z - self.left_z)
    }
}


pub fn start( bus: Bus ) {
    thread::spawn(move || {
        let mut gil = Gilrs::new();
        let mut state = State::new();

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
                    Event::AxisChanged(Axis::LeftTrigger2, v, _) => { state.left_z = v; Some(state.z_command()) },
                    Event::AxisChanged(Axis::RightTrigger2, v, _) => { state.right_z = v; Some(state.z_command()) },
                    _ => None,
                };

                if cmd == Some(Command::ManualControlReset) {
                    state = State::new();
                }

                match cmd {
                    Some(c) => drop(bus.sender.try_send(Message::Command(c))),
                    None => (),
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });
}
