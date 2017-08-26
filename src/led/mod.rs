//! LED lighting for the 'bot

use std::thread;
use botcomm::BotComm;


pub fn start(comm: &BotComm) {
    let comm = comm.try_clone().unwrap();
    thread::spawn(move || {
        // let anim = make_animators(comm);
        
        // let winch = Animator::new(comm.winch_leds(n), winch_led_model(n));
        // let 


    });
}





    // use tokio_core::reactor;
    // use tokio_periodic::PeriodicTimer;
    // use std::{io, thread, iter};
    // use std::time::Duration;
    // use std::io::Error;
    // use std::net::SocketAddr;
    // use std::sync::Mutex;
    // use futures::{Future, Stream, Sink, StartSend, Poll, Async, AsyncSink};
    // use futures::sync::mpsc;
    // use cgmath::{Point3, Rotation, Rotation3, Basis3, Rad, Angle};


    // #[derive(Clone, PartialEq, Debug)]
    // enum LedInfo {
    //     FlyerRing { loc: Point3<f64> },
    //     FlyerTop { dir: i32, loc: f64 },
    //     Winch { dir: i32, loc: f64 }
    // }


    // fn proportion(item: i32, size: i32) -> f64 {
    //     item as f64 / (size - 1) as f64
    // }

    // fn flyer_ledinfo() -> Vec<LedInfo> {
    //     // Short navigation LED strips on top
    //     let top = (0..3).flat_map(|dir| {
    //         const SIZE :i32 = 6;
    //         (0..SIZE).map(move |n| LedInfo::FlyerTop { dir, loc: proportion(n, SIZE) })
    //     });

    //     // Multiple full rings near the camera
    //     let rings = (0..2).flat_map(|level| {
    //         const SIZE :i32 = 36;
    //         let z = proportion(level - 1, 2);
    //         (0..SIZE).map(move |n| {
    //             let axis = Point3::new(1.0, 0.0, z);
    //             let angle = Rad::full_turn() * proportion(n, SIZE);
    //             let r: Basis3<f64> = Rotation3::from_angle_z(angle);
    //             let loc = r.rotate_point(axis); 
    //             LedInfo::FlyerRing { loc }
    //         })
    //     });

    //     top.chain(rings).collect()
    // }

    // fn winch_ledinfo(dir: i32) -> Vec<LedInfo> {
    //     // Short LED strips on the side of each winch
    //     const SIZE :i32 = 8;
    //     (0..SIZE).map(move |n| LedInfo::Winch { dir, loc: proportion(n, SIZE) }).collect()
    // }


    // #[derive(Clone, PartialEq, Eq, Debug)]
    // struct LedPixel {
    //     brightness: u8,
    //     red: u8,
    //     green: u8,
    //     blue: u8
    // }

    // enum AnimMode {
    //     Off
    // }

    // struct SharedState {
    //     mode: AnimMode,
    // }

    // pub struct Animator {
    //     state: Mutex<SharedState>,
    // }


    // struct TestSink;

    // impl Sink for TestSink {
    //     type SinkItem = Vec<u8>;
    //     type SinkError = ();

    //     fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
    //         println!("Sunk: {:?}", item);
    //         Ok(AsyncSink::Ready)
    //     }

    //     fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
    //         Ok(Async::Ready(()))
    //     }

    //     fn close(&mut self) -> Poll<(), Self::SinkError> {
    //         Ok(().into())
    //     }
    // }

    // pub fn try_it() {
    //     let leds = flyer_ledinfo();
    //     let sink = TestSink {};
    //     let (tx, rx) = mpsc::channel(10);
    //     let ani = animate(leds, tx);

    //     let mut core = reactor::Core::new().unwrap();
    //     let handle = core.handle();
    //     core.run(sink.send_all(rx));
    // }


    // fn frame_timer(handle: &reactor::Handle, fps: f64) -> PeriodicTimer {
    //     let t = PeriodicTimer::new(&handle).unwrap();
    //     t.reset(Duration::new(0, (1e9 / fps) as u32)).unwrap();
    //     t
    // }


    // fn animate(leds: Vec<LedInfo>, sink: mpsc::Sender<Vec<u8>>) -> Animator {
    //     const FPS: f64 = 100.0;
    //     let state = Mutex::new(SharedState {
    //         mode: AnimMode::Off
    //     });

    //     let thr = thread::spawn(|| {
    //         let mut core = reactor::Core::new().unwrap();
    //         let handle = core.handle();
    //         let frames = frame_timer(&handle, FPS).and_then(|x| {
    //             println!("item {:?}", x);
    //             // Ok(vec![LedPixel { brightness: 0, red: 0, green: 0, blue: 0 }])
    //             Ok(vec![1,2,3])
    //         });
            
    //         let panic_sink = sink.sink_map_err(|e| panic!("{}", e));
    //         let panic_stream = frames.map_err(|e| panic!("{}", e));
    //         core.run(panic_sink.send_all(panic_stream));
    //     });

    //     Animator { state }
    // }



    // /*
    // pub struct Animator {
    //     target_fps: f64,
    //     actual_fps: f64,
    //     frame: u64
    // }




    // impl Animator {

    //     pub fn new(handle: &reactor::Handle, fps: f64) -> Result<Animator, Error> {

    //     }

    // }
    // */

    // //   let mut frame = 0;

    // //     loop {
    // //         const PACKET_SIZE :usize = MAX_LEDS * 4 + 1;
    // //         let mut packet :[u8; PACKET_SIZE] = [0; PACKET_SIZE];

    // // 	packet[0] = MSG_LEDS;

    // //         let blips = [
    // //             (frame as f64 * 0.02,  (0.8, 1.0, 0.2)),
    // //             (frame as f64 * 0.03,  (0.2, 0.6, 0.9)),
    // //             (-frame as f64 * 0.01, (0.9, 0.3, 0.2)),
    // //         ];

    // //         for n in 0..(MAX_LEDS-1) {

    // //             let mut r :f64 = 0.0;
    // //             let mut g :f64 = 0.0;
    // //             let mut b :f64 = 0.0;

    // //             for blip in blips.iter() {
    // //                 let br = (n as f64 / 30.0 - blip.0).sin().powf(40.0);
    // //                 let color = blip.1;
    // //                 r = r + br * color.0;
    // //                 g = g + br * color.1;
    // //                 b = b + br * color.2;
    // //             }

    // //             let bright :u8 = ((r + g + b).powf(4.5) * 4.0 + 1.0).min(31.0).max(0.0) as u8;
    // //             let r :u8 = (0.5 + 255.0 * r).min(255.0).max(0.0) as u8;
    // //             let g :u8 = (0.5 + 255.0 * g).min(255.0).max(0.0) as u8;
    // //             let b :u8 = (0.5 + 255.0 * b).min(255.0).max(0.0) as u8;

    // //             packet[1+n*4] = 0xE0 | bright;
    // //             packet[2+n*4] = r;
    // //             packet[3+n*4] = g;
    // //             packet[4+n*4] = b;
    // //         }

    // //         s.send_to(&packet, dest).expect("send is sad");
    // //         frame = frame + 1;
    // //     }
    // // }

