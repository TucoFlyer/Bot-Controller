use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};
use botcomm::BotComm;
use led::apa102::APA102Pixel;
use led::shader::{Shader, LightEnvironment, PixelMapping};
use led::models::LEDModel;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

const FRAME_RATE : u32 = 120;

pub struct LightAnimator {
    env_sender: SyncSender<Option<LightEnvironment>>,
    last_sent: Option<LightEnvironment>,
}

impl LightAnimator {
    pub fn start(comm: &BotComm) -> LightAnimator {
        let last_sent = None;
        let (env_sender, env_recv) = sync_channel(128);
        let comm = comm.try_clone().unwrap();
        thread::spawn(move || {
            let mut anim = AnimatorThread::new(&comm, env_recv);
            loop {
                anim.frame();
            }
        });
        LightAnimator { env_sender, last_sent }
    }

    pub fn update(&mut self, env: LightEnvironment) {
        let env = Some(env);
        if self.last_sent != env {
            if self.env_sender.try_send(env.clone()).is_ok() {
                self.last_sent = env;
            }
        }
    }
}

struct AnimatorThread<'a> {
    env: Option<LightEnvironment>,
    recv: Receiver<Option<LightEnvironment>>,
    model: LEDModel<'a>,
    shader: Shader,
    last_frame_timestamp: Option<Instant>,
}

impl<'a> AnimatorThread<'a> {
    fn new(comm: &'a BotComm, recv: Receiver<Option<LightEnvironment>>) -> AnimatorThread<'a> {
        AnimatorThread {
            env: None,
            recv,
            model: LEDModel::new(comm),
            shader: Shader::new(),
            last_frame_timestamp: None,
        }
    }

    fn frame(&mut self) {
        let frame_duration = Duration::new(0, 1000000000 / FRAME_RATE);
        self.last_frame_timestamp = match self.last_frame_timestamp {
            None => {
                // First frame, new time reference
                Some(Instant::now())
            },
            Some(last_timestamp) => {
                let next_at = last_timestamp + frame_duration;
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

        // Receive all pending env updates, keep only the last
        match self.recv.try_iter().last() {
            Some(opt) => { self.env = opt; },
            None => {}
        }

        // Skip frames where we have no light environment
        match self.env {
            None => {}
            Some(ref env) => {
                self.shader.step(env, 1.0 / (FRAME_RATE as f64));
                for i in 0..self.model.vec.len() {
                    let buf = self.render(&self.model.vec[i].pixels, env);
                    drop(self.model.vec[i].writer.write(&buf));
                }
            }
        }
    }

    fn render(&self, mapping: &Vec<PixelMapping>, env: &LightEnvironment) -> Vec<u8> {
        let mut buf = Vec::new();
        for pixel_mapping in mapping.iter() {
            let pix : APA102Pixel = self.shader.pixel(env, pixel_mapping).to_pixel();
            pix.push_to_vec(&mut buf);
        }
        buf
    }
}
