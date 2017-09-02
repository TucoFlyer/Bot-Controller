use palette::pixel::RgbPixel;
use num_traits::Float;
use num_traits::cast::{ToPrimitive, FromPrimitive};
use bincode;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct APA102Pixel {
    pub header: u8,
    pub g: u8,
    pub b: u8,
    pub r: u8,
}

impl APA102Pixel {
    pub fn push_to_vec(&self, vec: &mut Vec<u8>) {
        bincode::serialize_into(vec, self, bincode::Infinite).unwrap()
    }
}

impl<T: Float + ToPrimitive + FromPrimitive> RgbPixel<T> for APA102Pixel {
    fn from_rgba(red: T, green: T, blue: T, _alpha: T) -> APA102Pixel {
        let red = red.to_f64().unwrap();
        let green = green.to_f64().unwrap();
        let blue = blue.to_f64().unwrap();

        // Scale the current source according to overall brightness too
        let brightness = (red*red + green*green + blue*blue).sqrt();

        // These LEDs seem to turn completely off if the red PWM channel is zero!? Avoid that.
        let red_min = if green > 0.0 || blue > 0.0 { 1.0 } else { 0.0 }; 

        APA102Pixel {
            header: 0xE0 | (brightness * 31.0).round().max(1.0).min(31.0) as u8,
            r: (red   * 255.0).round().max(red_min).min(255.0) as u8,
            g: (green * 255.0).round().max(0.0).min(255.0) as u8,
            b: (blue  * 255.0).round().max(0.0).min(255.0) as u8,
        }
    }

    fn to_rgba(&self) -> (T, T, T, T) {
        (
            T::from_f64((self.r as f64) / 255.0).unwrap(),
            T::from_f64((self.g as f64) / 255.0).unwrap(),
            T::from_f64((self.b as f64) / 255.0).unwrap(),
            T::one()
        )
    }
}
