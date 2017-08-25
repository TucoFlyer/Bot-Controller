extern crate bitflags;
extern crate palette;
extern crate cgmath;
extern crate kdtree;
extern crate serde_json;

pub mod layout;
pub mod particle;


use palette::{Rgb, Hsl, RgbHue, IntoColor};


pub fn render_frame(timer: f64) -> Vec<[u8;3]> {
    println!("t = {:?}", timer);

    (0..200).map(|pixel| {
        let x = pixel as f64 * 0.1 + timer * 4.0;
        Hsl::new(RgbHue::from_radians(x), 0.5, 0.5).into_rgb().to_pixel()
    }).collect()
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
