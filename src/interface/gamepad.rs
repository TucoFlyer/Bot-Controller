//! Bot control via a local gamepad

use bus::{Bus, Command, Message, ManualControlAxis};
use config::ControllerMode;
use gilrs::{Event, Button, Axis, Gilrs};
use std::thread;
use std::time::Duration;


struct State {
    left_enable: bool,
    right_enable: bool,
    left_z: f32,
    right_z: f32,
    rel_x: f32,
    rel_y: f32,
    cam_x: f32,
    cam_y: f32,
}

impl State {
    fn new() -> State {
        State {
            left_enable: false,
            right_enable: false,
            left_z: 0.0,
            right_z: 0.0,
            rel_x: 0.0,
            rel_y: 0.0,
            cam_x: 0.0,
            cam_y: 0.0,
        }
    }

    fn reset(&mut self) {
        *self = State::new();
    }

    fn x_command(self: &State) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeX, self.rel_x as f64)
    }

    fn y_command(self: &State) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeY, self.rel_y as f64)
    }

    fn z_command(self: &State) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeZ, (self.right_z - self.left_z) as f64)
    }

    fn pitch_command(self: &State) -> Command {
        Command::ManualControlValue(ManualControlAxis::CameraPitch, self.cam_y as f64)
    }

    fn yaw_command(self: &State) -> Command {
        Command::ManualControlValue(ManualControlAxis::CameraYaw, self.cam_x as f64)
    }

    fn is_enabled(self: &State) -> bool {
        self.left_enable || self.right_enable
    }
}

fn send_command(bus: &Bus, cmd: Command) {
    drop(bus.sender.try_send(Message::Command(cmd).timestamp()))
}

fn send_reset(bus: &Bus) {
    send_command(bus, Command::ManualControlReset)
}

fn send_complete(bus: &Bus, state: &State) {
    if state.is_enabled() {
        send_command(bus, state.x_command());
        send_command(bus, state.y_command());
        send_command(bus, state.z_command());
        send_command(bus, state.pitch_command());
        send_command(bus, state.yaw_command());
    } else {
        send_reset(bus);
    }
}

pub fn start( bus: &Bus ) {
    let bus = bus.clone();
    thread::spawn(move || {
        let mut gil = Gilrs::new();
        let mut state = State::new();

        loop {
            for (_id, event) in gil.poll_events() {
                match event {

                    Event::Connected => { state.reset(); send_reset(&bus) },
                    Event::Disconnected => { state.reset(); send_reset(&bus) },

                    Event::ButtonPressed(Button::LeftTrigger, _) => { state.left_enable = true; send_complete(&bus, &state) },
                    Event::ButtonReleased(Button::LeftTrigger, _) => { state.left_enable = false; send_complete(&bus, &state) },
                    Event::ButtonPressed(Button::RightTrigger, _) => { state.right_enable = true; send_complete(&bus, &state) },
                    Event::ButtonReleased(Button::RightTrigger, _) => { state.right_enable = false; send_complete(&bus, &state) },

                    Event::ButtonPressed(Button::East, _) => send_command(&bus, Command::SetMode(ControllerMode::Halted)),
                    Event::ButtonPressed(Button::South, _) => send_command(&bus, Command::SetMode(ControllerMode::ManualFlyer)),
                    Event::ButtonPressed(Button::West, _) => send_command(&bus, Command::SetMode(ControllerMode::ManualWinch(0))),

                    Event::AxisChanged(Axis::RightStickX, v, _) => { state.cam_x = v; if state.is_enabled() { send_command(&bus, state.yaw_command()) }},
                    Event::AxisChanged(Axis::RightStickY, v, _) => { state.cam_y = v; if state.is_enabled() { send_command(&bus, state.pitch_command()) }},

                    Event::AxisChanged(Axis::LeftStickX, v, _) => { state.rel_x = v; if state.is_enabled() { send_command(&bus, state.x_command()) }},
                    Event::AxisChanged(Axis::LeftStickY, v, _) => { state.rel_y = v; if state.is_enabled() { send_command(&bus, state.y_command()) }},
                    Event::AxisChanged(Axis::LeftTrigger2, v, _) => { state.left_z = v; if state.is_enabled() { send_command(&bus, state.z_command()) }},
                    Event::AxisChanged(Axis::RightTrigger2, v, _) => { state.right_z = v; if state.is_enabled() { send_command(&bus, state.z_command()) }},

                    _ => (),
                };
            }

            thread::sleep(Duration::from_millis(10));
        }
    });
}
