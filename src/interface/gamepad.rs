//! Bot control via a local gamepad

use message::{Command, Message, ManualControlAxis, CameraOutput};
use controller::ControllerPort;
use config::{SharedConfigFile, ControllerMode};
use gilrs::{Event, Button, Axis, Gilrs};
use std::thread;
use std::time::Duration;

struct State {
    left_enable: bool,
    right_enable: bool,
    gimbal_modifier: bool,
    recording_modifier: bool,
    livestream_modifier: bool,
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
            gimbal_modifier: false,
            livestream_modifier: false,
            recording_modifier: false,
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

    fn x_command(&self) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeX, self.rel_x)
    }

    fn y_command(&self) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeY, self.rel_y)
    }

    fn z_command(&self) -> Command {
        Command::ManualControlValue(ManualControlAxis::RelativeZ, self.right_z - self.left_z)
    }

    fn pitch_command(&self) -> Command {
        Command::ManualControlValue(ManualControlAxis::CameraPitch, self.cam_y)
    }

    fn yaw_command(&self) -> Command {
        Command::ManualControlValue(ManualControlAxis::CameraYaw, self.cam_x)
    }

    fn on_off_command(&self, enabled: bool) -> Option<Command> {
        if self.gimbal_modifier {
            Some(Command::GimbalMotorEnable(enabled))
        } else if self.recording_modifier {
            Some(Command::CameraOutputEnable(CameraOutput::LocalRecording, enabled))
        } else if self.livestream_modifier {
            Some(Command::CameraOutputEnable(CameraOutput::LiveStream, enabled))
        } else {
            None
        }
    }

    fn is_enabled(&self) -> bool {
        self.left_enable || self.right_enable
    }
}

fn send_command(c: &ControllerPort, cmd: Command) {
    c.send(Message::Command(cmd).timestamp());
}

fn send_reset(c: &ControllerPort) {
    send_command(c, Command::ManualControlReset);
}

fn send_on_off_command(c: &ControllerPort, state: &State, enabled: bool) {
    if let Some(cmd) = state.on_off_command(enabled) {
        send_command(&c, cmd);
    }
}

fn send_complete(c: &ControllerPort, state: &State) {
    if state.is_enabled() {
        send_command(c, state.x_command());
        send_command(c, state.y_command());
        send_command(c, state.z_command());
        send_command(c, state.pitch_command());
        send_command(c, state.yaw_command());
    } else {
        send_reset(c);
    }
}

pub fn start(config: &SharedConfigFile, c: &ControllerPort) {
    let config = config.clone();
    let c = c.clone();
    thread::Builder::new().name("Gamepad".into()).spawn(move || {
        let mut gil = Gilrs::new();
        let mut state = State::new();

        loop {
            for (_id, event) in gil.poll_events() {
                match event {

                    Event::Connected => { state.reset(); send_reset(&c) },
                    Event::Disconnected => { state.reset(); send_reset(&c) },

                    Event::ButtonPressed(Button::LeftTrigger, _) => { state.left_enable = true; send_complete(&c, &state) },
                    Event::ButtonReleased(Button::LeftTrigger, _) => { state.left_enable = false; send_complete(&c, &state) },
                    Event::ButtonPressed(Button::RightTrigger, _) => { state.right_enable = true; send_complete(&c, &state) },
                    Event::ButtonReleased(Button::RightTrigger, _) => { state.right_enable = false; send_complete(&c, &state) },

                    Event::ButtonPressed(Button::DPadUp, _) => { state.gimbal_modifier = true; },
                    Event::ButtonReleased(Button::DPadUp, _) => { state.gimbal_modifier = false; },
                    Event::ButtonPressed(Button::DPadLeft, _) => { state.recording_modifier = true; },
                    Event::ButtonReleased(Button::DPadLeft, _) => { state.recording_modifier = false; },
                    Event::ButtonPressed(Button::DPadRight, _) => { state.livestream_modifier = true; },
                    Event::ButtonReleased(Button::DPadRight, _) => { state.livestream_modifier = false; },

                    Event::ButtonPressed(Button::Select, _) => send_on_off_command(&c, &state, false),
                    Event::ButtonPressed(Button::Start, _) => send_on_off_command(&c, &state, true),

                    Event::ButtonPressed(Button::West, _) => send_command(&c, Command::SetMode(ControllerMode::ManualFlyer)),
                    Event::ButtonPressed(Button::East, _) => send_command(&c, Command::SetMode(ControllerMode::Halted)),
                    Event::ButtonPressed(Button::South, _) => send_command(&c, Command::SetMode(ControllerMode::Normal)),
                    Event::ButtonPressed(Button::North, _) => { send_command(&c, Command::SetMode( match config.get_latest().mode {
                        ControllerMode::ManualWinch(n) => ControllerMode::ManualWinch((n + 1) % config.get_latest().winches.len()),
                        _ => ControllerMode::ManualWinch(0),
                    }))},

                    Event::AxisChanged(Axis::RightStickX, v, _) => { state.cam_x = v; if state.is_enabled() { send_command(&c, state.yaw_command()) }},
                    Event::AxisChanged(Axis::RightStickY, v, _) => { state.cam_y = v; if state.is_enabled() { send_command(&c, state.pitch_command()) }},

                    Event::AxisChanged(Axis::LeftStickX, v, _) => { state.rel_x = v; if state.is_enabled() { send_command(&c, state.x_command()) }},
                    Event::AxisChanged(Axis::LeftStickY, v, _) => { state.rel_y = v; if state.is_enabled() { send_command(&c, state.y_command()) }},
                    Event::AxisChanged(Axis::LeftTrigger2, v, _) => { state.left_z = v; if state.is_enabled() { send_command(&c, state.z_command()) }},
                    Event::AxisChanged(Axis::RightTrigger2, v, _) => { state.right_z = v; if state.is_enabled() { send_command(&c, state.z_command()) }},

                    _ => (),
                };
            }

            thread::sleep(Duration::from_millis(10));
        }
    }).unwrap();
}
