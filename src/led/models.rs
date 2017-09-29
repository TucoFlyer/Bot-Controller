use vecmath::*;
use std::env;
use led::ledshape::{LEDShapeTemplate, to_point_cloud_file};
use led::shader::{PixelMapping, PixelUsage};
use botcomm::{BotComm, LEDWriter};

fn winch(id: usize) -> Vec<PixelMapping> {
	let mut model = Vec::new();

	let side_strip = LEDShapeTemplate {
		usage: PixelUsage::Winch(id),
		spacing: 1.0 / 60.0,
		count: 7,
	};

	let left_center = [-0.06, 0.0, 0.0];
	let right_center = [0.06, 0.0, 0.0];

	let vertical_direction = [0.0, 0.0, 1.0];

	side_strip.line(&mut model, left_center, vertical_direction);
	side_strip.line(&mut model, right_center, vertical_direction);

	model
}

fn flyer() -> Vec<PixelMapping> {
	let mut model = Vec::new();

	let top_strip = LEDShapeTemplate {
		usage: PixelUsage::FlyerTop,
		spacing: 1.0 / 144.0,
		count: 7,
	};

	let ring_strip = LEDShapeTemplate {
		usage: PixelUsage::FlyerRing,
		spacing: 1.0 / 144.0,
		count: 36,
	};

	let top_center = [0.0, 0.0, 0.45];
	let top_radius = [0.07, 0.0, 0.0];

	let upper_ring = [0.0, 0.0, 0.015];
	let middle_ring = [0.0, 0.0, 0.0];
	let lower_ring = [0.0, 0.0, -0.015];
	let ring_normal = [0.0, 0.0, 1.0];

	const NUM_STRIPS : usize = 4;
	for index in 0..NUM_STRIPS {
		let theta = (index as f64) / (NUM_STRIPS as f64) * TAU;
		let mat = rotation_matrix([0.0, 0.0, 1.0], theta);
		let radius = row_mat3x4_transform_vec3(mat, top_radius);
		top_strip.line(&mut model, vec3_add(top_center, radius), radius);
	}

	ring_strip.circle(&mut model, upper_ring, ring_normal, [1.0, 0.0, 0.0]);
	ring_strip.circle(&mut model, middle_ring, ring_normal, [-1.0, 0.0, 0.0]);
	ring_strip.circle(&mut model, lower_ring, ring_normal, [1.0, 0.0, 0.0]);

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
	pub fn new(comm: &'a BotComm) -> LEDModel<'a> {
		let mut vec = Vec::new();

		vec.push(LEDWriterMapping {
			writer: comm.flyer_leds(),
			pixels: flyer(),
		});

		for id in 0..comm.num_winches() {
			vec.push(LEDWriterMapping {
				writer: comm.winch_leds(id),
				pixels: winch(id),
			});
		}

		if let Ok(_) = env::var("SAVE_LED_MODEL") {
			to_point_cloud_file(&flyer(), "flyer_leds.xyz").unwrap();
			to_point_cloud_file(&winch(0), "winch_leds.xyz").unwrap();
			println!("Saved LED models as point cloud files");
		}

		LEDModel { vec }
	}
}
