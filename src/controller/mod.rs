//! Controller thread, responding to commands and status updates, generating motor control packets

mod manual;
mod velocity;
mod winch;
mod state;
mod scheduler;

use bus::*;
use std::thread;
use config::ConfigFile;
use botcomm::BotSender;
use self::state::ControllerState;
use self::scheduler::Scheduler;
use led::LightAnimator;
use std::time::{Duration, Instant};
use overlay::{DrawingContext, VIDEO_HZ};

pub fn start(bus: &Bus, comm: &BotSender, cf: ConfigFile) {
    let bus = bus.clone();
    let comm = comm.try_clone().unwrap();
    thread::spawn(move || {
        let mut controller = Controller::new(bus, comm, cf);
        loop {
            controller.poll();
        }
    });
}

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

struct Controller {
    bus: Bus,
    comm: BotSender,
    cf: ConfigFile,
    state: ControllerState,
    sched: Scheduler,
    per_tick: IntervalTimer,
    per_video_frame: IntervalTimer,
    draw: DrawingContext,
}

impl Controller {
    fn new(bus: Bus, comm: BotSender, cf: ConfigFile) -> Controller {
        let lights = LightAnimator::start(&cf.config.lighting.animation, &comm);
        let state = ControllerState::new(&cf.config, lights);
        Controller {
            bus, comm, cf, state,
            sched: Scheduler::new(),
            per_tick: IntervalTimer::new(TICK_HZ),
            per_video_frame: IntervalTimer::new(VIDEO_HZ),
            draw: DrawingContext::new(),
        }
    }

    fn config_changed(self: &mut Controller) {
        *self.bus.config.lock().unwrap() = self.cf.config.clone();
        drop(self.bus.sender.try_send(Message::ConfigIsCurrent(self.cf.config.clone()).timestamp()));
        self.cf.save_async();
        self.state.config_changed(&self.cf.config);
    }

    fn poll(self: &mut Controller) {
        if let Ok(ts_msg) = self.bus.receiver.recv() {
            let timestamp = ts_msg.timestamp;
            match ts_msg.message {

                Message::UpdateConfig(updates) => {
                    // Merge a freeform update into the configuration, and save it.
                    // Errors here go right to the console, since errors caused by a
                    // client should have been detected earlier and sent to that client.
                    match self.cf.config.merge(updates) {
                        Err(e) => println!("Error in UpdateConfig from message bus: {}", e),
                        Ok(config) => {
                            self.cf.config = config;
                            self.config_changed();
                        }
                    }
                }

                Message::Command(Command::SetMode(mode)) => {
                    // The controller mode is part of the config, so this could be changed via UpdateConfig as well, but this option is strongly typed
                    if self.cf.config.mode != mode {
                        self.cf.config.mode = mode;
                        self.config_changed();
                    }
                }

                Message::WinchStatus(id, status) => {
                    drop(self.comm.winch_command(id, self.state.winch_control_loop(&self.cf.config, id, status)));
                },

                Message::FlyerSensors(sensors) => {
                    self.state.flyer_sensor_update(sensors);
                },

                Message::Command(Command::ManualControlValue(axis, value)) => {
                    self.state.manual.control_value(axis, value);
                },

                Message::Command(Command::ManualControlReset) => {
                    self.state.manual.control_reset();
                },

                _ => (),
            }

            if self.per_tick.poll() {
                self.state.every_tick(&self.cf.config);
            }

            if self.per_video_frame.poll() {
                self.draw.clear();
                self.state.draw_camera_overlay(&self.cf.config, &mut self.draw);
                let scene = self.draw.scene.drain(..).collect();
                drop(self.bus.sender.try_send(Message::CameraOverlayScene(scene).timestamp()));
            }

            if self.sched.poll_config_changes(timestamp, &mut self.cf.config) {
                self.config_changed();
            }
        }
    }
}
