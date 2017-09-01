use palette::pixel::RgbPixel;
use num_traits::Float;
use num_traits::cast::{ToPrimitive, FromPrimitive};

pub struct APA102Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub header: u8,
}

impl APA102Pixel {
    pub fn push_to_vec(&self, vec: &mut Vec<u8>) {
        vec.push(self.header);
        vec.push(self.g);
        vec.push(self.b);
        vec.push(self.r);
    }
}

impl<T: Float + ToPrimitive + FromPrimitive> RgbPixel<T> for APA102Pixel {
    fn from_rgba(red: T, green: T, blue: T, _alpha: T) -> APA102Pixel {
        let brightness = (red*red + green*green + blue*blue).sqrt();
        APA102Pixel {
            header: 0xE0 | (brightness.to_f64().unwrap() * 31.0).round().max(1.0).min(31.0) as u8,
            r: (red  .to_f64().unwrap() * 255.0).round().max(0.0).min(255.0) as u8,
            g: (green.to_f64().unwrap() * 255.0).round().max(0.0).min(255.0) as u8,
            b: (blue .to_f64().unwrap() * 255.0).round().max(0.0).min(255.0) as u8,
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
