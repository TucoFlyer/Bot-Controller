//! Controller thread, responding to commands and status updates, generating motor control packets

mod manual;
mod velocity;
mod winch;
mod state;
mod timer;
mod gimbal;
mod draw;

use message::*;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use bus::{Bus, BusReader};
use config::{SharedConfigFile, Config};
use botcomm::BotSocket;
use fygimbal::GimbalPort;
use self::state::ControllerState;
use self::timer::{ConfigScheduler, ControllerTimers};
use self::gimbal::GimbalController;
use led::LightAnimator;
use overlay::DrawingContext;

pub struct Controller {
    recv: Receiver<ControllerInput>,
    bus: Bus<TimestampedMessage>,
    port_prototype: ControllerPort,
    socket: BotSocket,
    shared_config: SharedConfigFile,
    local_config: Config,
    state: ControllerState,
    config_scheduler: ConfigScheduler,
    timers: ControllerTimers,
    draw: DrawingContext,
    gimbal_ctrl: GimbalController,
    gimbal_status: Option<GimbalControlStatus>,
}

enum ControllerInput {
    Message(TimestampedMessage),
    ReaderRequest(SyncSender<BusReader<TimestampedMessage>>),
}

#[derive(Clone)]
pub struct ControllerPort {
    sender: SyncSender<ControllerInput>,
}

impl ControllerPort {
    pub fn send(&self, msg: TimestampedMessage) {
        if self.sender.try_send(ControllerInput::Message(msg)).is_err() {
            println!("Controller input queue overflow!");
        }
    }

    pub fn add_rx(&self) -> BusReader<TimestampedMessage> {
        let (result_sender, result_recv) = sync_channel(1);
        drop(self.sender.try_send(ControllerInput::ReaderRequest(result_sender)));
        result_recv.recv().unwrap()
    }
}

impl Controller {
    pub fn new(config: &SharedConfigFile, socket: &BotSocket) -> Controller {

        const DEPTH : usize = 1024;

        let (sender, recv) = sync_channel(DEPTH);
        let bus = Bus::new(DEPTH);
        let port_prototype = ControllerPort { sender };

        let local_config = config.get_latest();
        let lights = LightAnimator::start(&local_config.lighting.animation, &socket);
        let state = ControllerState::new(&local_config, lights);

        Controller {
            recv,
            bus,
            port_prototype,
            state,
            local_config,
            socket: socket.try_clone().unwrap(),
            shared_config: config.clone(),
            config_scheduler: ConfigScheduler::new(),
            timers: ControllerTimers::new(),
            draw: DrawingContext::new(),
            gimbal_ctrl: GimbalController::new(),
            gimbal_status: None
        }
    }

    pub fn port(&self) -> ControllerPort {
        self.port_prototype.clone()
    }

    pub fn run(mut self, gimbal_port: GimbalPort) {
        println!("Running.");
        loop {
            self.poll(&gimbal_port);
        }
    }

    fn broadcast(&mut self, ts_msg: TimestampedMessage) {
        if self.bus.try_broadcast(ts_msg).is_err() {
            println!("Controller output bus overflow!");
        }
    }

    fn config_changed(&mut self) {
        self.shared_config.set(self.local_config.clone());
        let msg = Message::ConfigIsCurrent(self.local_config.clone());
        self.broadcast(msg.timestamp());
        self.state.config_changed(&self.local_config);
    }

    fn poll(&mut self, gimbal_port: &GimbalPort) {
        match self.recv.recv().unwrap() {

            ControllerInput::ReaderRequest(result_channel) => {
                // Never blocks, result_channel must already have room
                let rx = self.bus.add_rx();
                drop(result_channel.try_send(rx));
            }

            ControllerInput::Message(ts_msg) => {
                self.broadcast(ts_msg.clone());
                self.handle_message(ts_msg, gimbal_port);
            }
        }

        if self.timers.tick.poll() {
            self.state.every_tick(&self.local_config);

            let gimbal_status = self.gimbal_ctrl.tick(&self.local_config, gimbal_port, &self.state.tracked);
            self.gimbal_status = Some(gimbal_status.clone());
            self.broadcast(Message::GimbalControlStatus(gimbal_status).timestamp());

            if let Some(tracking_rect) = self.state.tracking_update(&self.local_config, 1.0 / TICK_HZ as f32) {
                self.broadcast(Message::CameraInitTrackedRegion(tracking_rect).timestamp());
            }
        }

        if self.timers.video_frame.poll() {
            self.render_overlay();
            let scene = self.draw.scene.drain(..).collect();
            self.broadcast(Message::CameraOverlayScene(scene).timestamp());
        }

        if self.config_scheduler.poll(&mut self.local_config) {
            self.config_changed();
        }
    }

    fn render_overlay(&mut self) {
        let config = &self.local_config;
        self.draw.clear();
        draw::mode_indicator(config, &mut self.draw);
        draw::detected_objects(config, &mut self.draw, &self.state.detected.1);
        draw::tracking_gains(config, &mut self.draw, &self.gimbal_status);
        draw::tracking_rect(config, &mut self.draw, &self.state.tracked, &self.state.manual);
        draw::gimbal_status(config, &mut self.draw, &self.gimbal_status);
        self.state.tracking_particles.render(config, &mut self.draw);
        draw::debug_text(config, &mut self.draw, format!("{:?}, {:?}", config.mode, self.gimbal_status));
    }

    fn handle_message(&mut self, ts_msg: TimestampedMessage, gimbal_port: &GimbalPort) {
        match ts_msg.message {

            Message::UpdateConfig(updates) => {
                // Merge a freeform update into the configuration, and save it.
                // Errors here go right to the console, since errors caused by a
                // client should have been detected earlier and sent to that client.
                match self.local_config.merge(updates) {
                    Err(e) => println!("Error in UpdateConfig from message bus: {}", e),
                    Ok(config) => {
                        self.local_config = config;
                        self.config_changed();
                    }
                }
            }

            Message::WinchStatus(id, status) => {
                let command = self.state.winch_control_loop(&self.local_config, id, status);
                drop(self.socket.winch_command(id, command));
            },

            Message::FlyerSensors(sensors) => {
                self.state.flyer_sensor_update(sensors);
            },

            Message::GimbalValue(val, _) => {
                self.gimbal_ctrl.value_received(val)
            },

            Message::Command(Command::CameraObjectDetection(obj)) => {
                self.state.camera_object_detection_update(obj);
                if let Some(tracking_rect) = self.state.tracking_update(&self.local_config, 0.0) {
                    self.broadcast(Message::CameraInitTrackedRegion(tracking_rect).timestamp());
                }
            },

            Message::Command(Command::CameraRegionTracking(tr)) => {
                self.state.camera_region_tracking_update(tr);
            },

            Message::Command(Command::SetMode(mode)) => {
                // The controller mode is part of the config, so this could be changed via UpdateConfig as well, but this option is strongly typed
                if self.local_config.mode != mode {
                    self.local_config.mode = mode;
                    self.config_changed();
                }
            },

            Message::Command(Command::GimbalMotorEnable(en)) => {
                gimbal_port.set_motor_enable(en);
            },

            Message::Command(Command::GimbalPacket(packet)) => {
                gimbal_port.send_packet(packet);
            },

            Message::Command(Command::GimbalValueWrite(data)) => {
                gimbal_port.write_value(data);
            },

            Message::Command(Command::GimbalValueRequests(reqs)) => {
                gimbal_port.request_values(reqs);
            },

            Message::Command(Command::ManualControlValue(axis, value)) => {
                self.state.manual.control_value(axis, value);
            },

            Message::Command(Command::ManualControlReset) => {
                self.state.manual.control_reset();
            },

            _ => (),
        }
    }
}
