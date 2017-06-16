use std::net::UdpSocket;

const MAX_LEDS :usize = 200;
const MSG_LEDS :u8 = 5;

fn main() {
    let dest = "127.0.0.1:9024";
    let s = UdpSocket::bind("0.0.0.0:0").expect("cant has socket");
    let mut frame = 0;

    loop {
        const PACKET_SIZE :usize = MAX_LEDS * 4 + 1;
        let mut packet :[u8; PACKET_SIZE] = [0; PACKET_SIZE];

	packet[0] = MSG_LEDS;

        let blips = [
            (frame as f64 * 0.02,  (0.8, 1.0, 0.2)),
            (frame as f64 * 0.03,  (0.2, 0.6, 0.9)),
            (-frame as f64 * 0.01, (0.9, 0.3, 0.2)),
        ];

        for n in 0..(MAX_LEDS-1) {

            let mut r :f64 = 0.0;
            let mut g :f64 = 0.0;
            let mut b :f64 = 0.0;

            for blip in blips.iter() {
                let br = (n as f64 / 30.0 - blip.0).sin().powf(40.0);
                let color = blip.1;
                r = r + br * color.0;
                g = g + br * color.1;
                b = b + br * color.2;
            }

            let bright :u8 = ((r + g + b).powf(4.5) * 4.0 + 1.0).min(31.0).max(0.0) as u8;
            let r :u8 = (0.5 + 255.0 * r).min(255.0).max(0.0) as u8;
            let g :u8 = (0.5 + 255.0 * g).min(255.0).max(0.0) as u8;
            let b :u8 = (0.5 + 255.0 * b).min(255.0).max(0.0) as u8;

            packet[1+n*4] = 0xE0 | bright;
            packet[2+n*4] = r;
            packet[3+n*4] = g;
            packet[4+n*4] = b;
        }

        s.send_to(&packet, dest).expect("send is sad");
        frame = frame + 1;
    }
}

