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

pub fn vec3_mix(a: Vector3<f64>, b: Vector3<f64>, scale: f64) -> Vector3<f64> {
	vec3_add(a, vec3_scale(vec3_sub(b, a), scale))
}