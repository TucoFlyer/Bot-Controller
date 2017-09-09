use vecmath::*;
use led::shader::{PixelMapping, PixelUsage};
use std::fs::File;
use std::io;
use std::path::Path;

pub struct LEDShapeTemplate {
	pub usage: PixelUsage,
	pub count: usize,
	pub spacing: f64,
}

impl LEDShapeTemplate {
	pub fn line(self: &LEDShapeTemplate, model: &mut Vec<PixelMapping>, center: Vector3<f64>, tangent: Vector3<f64>) {
		let length = (self.count - 1) as f64 * self.spacing;
		let tangent = vec3_normalized(tangent);
		for index in 0..self.count {
			let alpha = (index as f64) / ((self.count - 1) as f64) - 0.5;
			model.push(PixelMapping {
				location: vec3_add(center, vec3_scale(tangent, alpha * length)),
				usage: self.usage.clone(),
			})
		}
	}

	pub fn circle(self: &LEDShapeTemplate, model: &mut Vec<PixelMapping>, center: Vector3<f64>, normal: Vector3<f64>) {
		let normal = vec3_normalized(normal);
		let circumference = self.count as f64 * self.spacing;
		let radius = circumference / TAU;
		let radius_vec = vec3_scale(arbitrary_perpendicular_vector(normal), radius);
		for index in 0..self.count {
			let theta = (index as f64) / (self.count as f64) * TAU;
			let mat = rotation_matrix(normal, theta);
			model.push(PixelMapping {
				location: vec3_add(center, row_mat3x4_transform_pos3(mat, radius_vec)),
				usage: self.usage.clone(),
			});
		}
	}
}

pub fn to_point_cloud(map: &[PixelMapping], w: &mut io::Write) -> io::Result<()> {
	for ref pixel in map {
		let xyz = pixel.location;
		writeln!(w, "{} {} {}", xyz[0], xyz[1], xyz[2])?
	}
	Ok(())
}

pub fn to_point_cloud_file<P: AsRef<Path>>(map: &[PixelMapping], path: P) -> io::Result<()> {
	let mut file = File::create(path)?;
	to_point_cloud(map, &mut file)
}
