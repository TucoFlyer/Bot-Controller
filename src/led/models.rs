use vecmath::*;
use led::shader::{PixelUsage, PixelMapping};
use botcomm::{BotSocket, LEDWriter};

fn winch(id: usize) -> Vec<PixelMapping> {
	let mut model = Vec::new();
	let format = [1,2,0];

	for minor_axis in [-1.0, 1.0].iter() {
		for led_index in 0..8 {
			let led_spacing = 1.0 / 60.0;
			let major_axis = led_spacing * led_index as f32;
			model.push(PixelMapping {
				format,
				usage: PixelUsage::Winch(id, major_axis, *minor_axis),
			});
		}
	}

	model
}

fn flyer() -> Vec<PixelMapping> {
	let mut model = Vec::new();
	let format = [2,1,0];

	for winch_id in [3, 0, 1, 2].iter() {
		for led_index in 0..7 {
			let led_spacing = 1.0 / 144.0;
			let major_axis = led_spacing * led_index as f32;
			model.push(PixelMapping {
				format,
				usage: PixelUsage::Winch(*winch_id, major_axis, 0.0),
			});
		}
	}

	for &(ring_offset, ring_z) in [
		(PI, 1.0),
		(0.0, 0.0),
		(PI, -1.0),
	].iter() {
		for led_index in 0..36 {
			let led_angle = led_index as f32 * -(PI * 2.0 / 36.0);
			model.push(PixelMapping {
				format,
				usage: PixelUsage::FlyerRing(led_angle + ring_offset, ring_z),
			});
		}
	}

	model
}

#[derive(Debug)]
pub struct LEDWriterMapping<'a> {
	pub writer: LEDWriter<'a>,
	pub pixels: Vec<PixelMapping>
}

#[derive(Debug)]
pub struct LEDModel<'a> {
	pub vec: Vec<LEDWriterMapping<'a>>,
}

impl<'a> LEDModel<'a> {
	pub fn new(socket: &'a BotSocket) -> LEDModel<'a> {
		let mut vec = Vec::new();

		vec.push(LEDWriterMapping {
			writer: socket.flyer_leds(),
			pixels: flyer(),
		});

		for id in 0..socket.num_winches() {
			vec.push(LEDWriterMapping {
				writer: socket.winch_leds(id),
				pixels: winch(id),
			});
		}

		LEDModel { vec }
	}
}
