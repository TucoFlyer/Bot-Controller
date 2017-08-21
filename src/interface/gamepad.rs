//! Bot control via a local gamepad

use bus::{Bus, Command, Message, ControllerMode, ManualControlAxis};
use gilrs::{Event, Button, Axis, Gilrs};
use std::thread;
use std::time::Duration;


struct State {
    left_enable: bool,
    right_enable: bool,
    left_z: f32,
    right_z: f32,
}

impl State {
    fn new() -> State {
        State {
            left_enable: false,
            right_enable: false,
            left_z: 0.0,
            right_z: 0.0
        }
    }

    fn z_command(self: &State) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeZ, self.right_z - self.left_z)
    }

    fn any_enable_button(self: &State) -> bool {
        self.left_enable || self.right_enable
    }

    fn if_enabled(self: &State, cmd: Command) -> Option<Command> {
        if self.any_enable_button() { Some(cmd) } else { None }
    }

    fn reset_if_disabled(self: &State) -> Option<Command> {
        if self.any_enable_button() { None } else { Some(Command::ManualControlReset) }
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

                    Event::ButtonPressed(Button::LeftTrigger, _) => { state.left_enable = true; None },
                    Event::ButtonReleased(Button::LeftTrigger, _) => { state.left_enable = false; state.reset_if_disabled() },
                    Event::ButtonPressed(Button::RightTrigger, _) => { state.right_enable = true; None },
                    Event::ButtonReleased(Button::RightTrigger, _) => { state.right_enable = false; state.reset_if_disabled() },

                    Event::ButtonPressed(Button::East, _) => Some(Command::SetMode(ControllerMode::Halted)),
                    Event::ButtonPressed(Button::South, _) => Some(Command::SetMode(ControllerMode::ManualFlyer)),
                    Event::ButtonPressed(Button::West, _) => Some(Command::SetMode(ControllerMode::ManualWinch(0))),

                    Event::AxisChanged(Axis::RightStickX, v, _) => state.if_enabled(Command::ManualControlValue(ManualControlAxis::CameraYaw, v)),
                    Event::AxisChanged(Axis::RightStickY, v, _) => state.if_enabled(Command::ManualControlValue(ManualControlAxis::CameraPitch, v)),
                    Event::AxisChanged(Axis::LeftStickX, v, _) => state.if_enabled(Command::ManualControlValue(ManualControlAxis::RelativeX, v)),
                    Event::AxisChanged(Axis::LeftStickY, v, _) => state.if_enabled(Command::ManualControlValue(ManualControlAxis::RelativeY, v)),
                    Event::AxisChanged(Axis::LeftTrigger2, v, _) => { state.left_z = v; state.if_enabled(state.z_command()) },
                    Event::AxisChanged(Axis::RightTrigger2, v, _) => { state.right_z = v; state.if_enabled(state.z_command()) },
                    _ => None,
                };

                if cmd == Some(Command::ManualControlReset) {
                    state = State::new();
                }

                match cmd {
                    Some(c) => drop(bus.sender.try_send(Message::Command(c).timestamp())),
                    None => (),
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });
}
