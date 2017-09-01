use palette::Rgb;
use num_traits::Float;

pub use vecmath_lib::*;
pub use std::f64::consts::PI;
pub const TAU : f64 = PI * 2.0;

pub fn rotation_matrix(normalized_axis: Vector3<f64>, angle: f64) -> Matrix3x4<f64> {
	let (x, y, z) = (normalized_axis[0], normalized_axis[1], normalized_axis[2]);
	let (sin, cos) = angle.sin_cos();
	let icos = 1.0 - cos;
    [
        [cos + x*x*icos, x*y*icos - z*sin, x*z*icos + y*sin, 0.0],
        [y*x*icos + z*sin, cos + y*y*icos, y*z*icos - x*sin, 0.0],
        [z*x*icos - y*sin, z*y*icos + x*sin, cos+z*z*icos, 0.0]
    ]
}

pub fn arbitrary_perpendicular_vector(v: Vector3<f64>) -> Vector3<f64> {
	if v[1].is_normal() && v[2].is_normal() {
		vec3_cross(v, [1.0, 0.0, 0.0])
	} else {
		vec3_cross(v, [0.0, 1.0, 0.0])
	}
}

pub fn vec3_to_rgb<T: Float>(v: Vector3<T>) -> Rgb<T> {
	Rgb::new(v[0], v[1], v[2])
}
