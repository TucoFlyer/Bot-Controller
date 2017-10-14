use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};
use botcomm::BotSocket;
use led::format::write_apa102_pixel;
use led::shader::{Shader, LightEnvironment, PixelMapping};
use led::models::LEDModel;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use config::LightAnimatorConfig;
use led::interpolate::serde_interpolate;

pub struct LightAnimator {
    env_sender: SyncSender<LightEnvironment>,
    last_sent: Option<LightEnvironment>,
}

impl LightAnimator {
    pub fn start(config: &LightAnimatorConfig, socket: &BotSocket) -> LightAnimator {
        let last_sent = None;
        let (env_sender, env_recv) = sync_channel(128);
        let socket = socket.try_clone().unwrap();
        let config = config.clone();
        thread::Builder::new().name("LightAnimator".into()).spawn(move || {
            let mut anim = AnimatorThread::new(config, &socket, env_recv);
            loop {
                anim.frame();
            }
        }).unwrap();
        LightAnimator {
            env_sender,
            last_sent,
        }
    }

    pub fn update(&mut self, env: LightEnvironment) {
        let is_different = match self.last_sent {
            None => true,
            Some(ref last_sent) => last_sent != &env,
        };
        if is_different && self.env_sender.try_send(env.clone()).is_ok() {
            self.last_sent = Some(env);
        }
    }
}

struct AnimatorThread<'a> {
    config: LightAnimatorConfig,
    env: Option<LightEnvironment>,
    recv: Receiver<LightEnvironment>,
    model: LEDModel<'a>,
    shader: Shader,
    interpolation_target: Option<LightEnvironment>,
    last_frame_timestamp: Option<Instant>,
}

impl<'a> AnimatorThread<'a> {
    fn new(config: LightAnimatorConfig, socket: &'a BotSocket, recv: Receiver<LightEnvironment>) -> AnimatorThread<'a> {
        AnimatorThread {
            config,
            recv,
            model: LEDModel::new(socket),
            shader: Shader::new(),
            env: None,
            last_frame_timestamp: None,
            interpolation_target: None,
        }
    }

    fn frame_duration(&self) -> Duration {
        Duration::new(0, (1e9 / self.config.frame_rate).round() as u32)
    }

    fn frame(&mut self) {
        self.last_frame_timestamp = match self.last_frame_timestamp {
            None => {
                // First frame, new time reference
                Some(Instant::now())
            },
            Some(last_timestamp) => {
                let next_at = last_timestamp + self.frame_duration();
                let now = Instant::now();
                if next_at <= now {
                    // Not keeping up, reset time reference
                    Some(now)
                } else {
                    thread::sleep(next_at.duration_since(now));
                    Some(next_at)
                }
            }
        };

        // Receive all pending env updates, keep only the last.
        self.interpolation_target = self.recv.try_iter().last().or_else(|| self.interpolation_target.clone());

        // Interpolate between LightEnvironments to prevent flicker and make transitions less abrupt
        self.env = match self.env {
            None => self.interpolation_target.clone(),
            Some(ref last_env) => Some(match self.interpolation_target {
                None => last_env.clone(),
                Some(ref target) => serde_interpolate(last_env, target, self.config.filter_param),
            }),
        };

        if let Some(ref env) = self.env {
            self.shader.step(env, 1.0 / self.config.frame_rate);
            for i in 0..self.model.vec.len() {
                let buf = self.render(&self.model.vec[i].pixels, env);
                drop(self.model.vec[i].writer.write(&buf));
            }
        }
    }

    fn render(&self, mapping: &Vec<PixelMapping>, env: &LightEnvironment) -> Vec<u8> {
        let mut buf = Vec::new();
        for pixel_mapping in mapping.iter() {
            write_apa102_pixel(&mut buf, self.shader.pixel(env, pixel_mapping)).unwrap();
        }
        buf
    }
}
