extern crate opc;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_periodic;
extern crate futures;
extern crate ledshader;

use opc::{OpcCodec, Message, Command};
use futures::{Stream, Future, Sink, future, stream};
use tokio_io::AsyncRead;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_periodic::PeriodicTimer;
use std::io;
use std::time::Duration;

const FPS :f64 = 100.0;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let remote_addr = "127.0.0.1:7890".parse().unwrap();

    let work = TcpStream::connect(&remote_addr, &handle)
        .and_then(|socket| {

            let transport = socket.framed(OpcCodec);

            let timer = PeriodicTimer::new(&handle).unwrap();
            timer.reset(Duration::new(0, (1e9 / FPS) as u32)).unwrap();

            let frame_numbers = stream::iter((0..std::u64::MAX).cycle().map(|n| Ok::<_,io::Error>(n)));

            let frames = timer.zip(frame_numbers).and_then(|(_, frame_number)| {
                future::ok(ledshader::render_frame(frame_number as f64 / FPS))
            });

            let messages = frames.and_then(|pixels| {
                let pixel_msg = Message {
                    channel: 0,
                    command: Command::SetPixelColors { pixels }
                };
                future::ok::<_,io::Error>(pixel_msg)
            });

            transport.send_all(messages)
        });

    core.run(work).unwrap();
}
