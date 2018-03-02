use vecmath::Vector3;
use std::io;

pub fn write_apa102_pixel(wr: &mut io::Write, color: Vector3<f32>) -> io::Result<(usize)> {
    let c0 = color[0];
    let c1 = color[1];
    let c2 = color[2];

    // Scale the current source according to overall brightness too
    let brightness = (c0*c0 + c1*c1 + c2*c2).sqrt();

    let header = 0xE0 | (brightness * 31.0).round().max(1.0).min(31.0) as u8;
    let c0 = (c0 * 255.0).round().max(1.0).min(255.0) as u8;
    let c1 = (c1 * 255.0).round().max(1.0).min(255.0) as u8;
    let c2 = (c2 * 255.0).round().max(1.0).min(255.0) as u8;

    let packet = [header, c0, c1, c2];
    wr.write(&packet)
}
