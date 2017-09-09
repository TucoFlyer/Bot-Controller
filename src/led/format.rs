use vecmath::Vector3;
use std::io;

pub fn write_apa102_pixel(wr: &mut io::Write, color: Vector3<f64>) -> io::Result<(usize)> {
    let red = color[0];
    let green = color[1];
    let blue = color[2];

    // Scale the current source according to overall brightness too
    let brightness = (red*red + green*green + blue*blue).sqrt();

    let header = 0xE0 | (brightness * 31.0).round().max(1.0).min(31.0) as u8;
    let green = (green * 255.0).round().max(0.0).min(255.0) as u8;
    let blue = (blue * 255.0).round().max(0.0).min(255.0) as u8;

    // These LEDs seem to turn completely off if the red PWM channel is zero!? Avoid that.
    let red_min = if green > 0 || blue > 0 { 1.0 } else { 0.0 }; 
    let red = (red * 255.0).round().max(red_min).min(255.0) as u8;

    let packet = [header, green, blue, red];
    wr.write(&packet)
}
