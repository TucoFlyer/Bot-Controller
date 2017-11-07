#![allow(dead_code)]

use rand::{thread_rng, Rng};
use num::traits::{Float, zero, one};
use vecmath_lib::traits::Sqrt;
use num::FromPrimitive;

pub use vecmath_lib::*;
pub use std::f32::consts::PI;
pub const TAU : f32 = PI * 2.0;

pub fn rotation_matrix<T: Float>(normalized_axis: Vector3<T>, angle: T) -> Matrix3x4<T> {
	let (x, y, z) = (normalized_axis[0], normalized_axis[1], normalized_axis[2]);
	let (sin, cos) = angle.sin_cos();
	let icos = one::<T>() - cos;
    [
        [cos + x*x*icos, x*y*icos - z*sin, x*z*icos + y*sin, zero()],
        [y*x*icos + z*sin, cos + y*y*icos, y*z*icos - x*sin, zero()],
        [z*x*icos - y*sin, z*y*icos + x*sin, cos+z*z*icos, zero()]
    ]
}

pub fn vec3_mix<T: Float>(a: Vector3<T>, b: Vector3<T>, scale: T) -> Vector3<T> {
	vec3_add(a, vec3_scale(vec3_sub(b, a), scale))
}

pub fn rect_topleft<T: Float>(r: Vector4<T>) -> Vector2<T> {
	[rect_left(r), rect_top(r)]
}

pub fn rect_bottomleft<T: Float>(r: Vector4<T>) -> Vector2<T> {
	[rect_left(r), rect_bottom(r)]
}

pub fn rect_topright<T: Float>(r: Vector4<T>) -> Vector2<T> {
	[rect_right(r), rect_top(r)]
}

pub fn rect_bottomright<T: Float>(r: Vector4<T>) -> Vector2<T> {
	[rect_right(r), rect_bottom(r)]
}

fn half<T: FromPrimitive>() -> T {
	T::from_f32(0.5).unwrap()
}

pub fn rect_center<T: Float + FromPrimitive>(r: Vector4<T>) -> Vector2<T> {
	[r[0] + r[2]*half(), r[1] + r[3]*half()]
}

pub fn rect_area<T: Float>(r: Vector4<T>) -> T {
	r[2] * r[3]
}

pub fn rect_top<T: Float>(r: Vector4<T>) -> T {
	r[1]
}

pub fn rect_left<T: Float>(r: Vector4<T>) -> T {
	r[0]
}

pub fn rect_right<T: Float>(r: Vector4<T>) -> T {
	r[0] + r[2]
}

pub fn rect_bottom<T: Float>(r: Vector4<T>) -> T {
	r[1] + r[3]
}

pub fn rect_offset<T: Float>(r: Vector4<T>, o: T) -> Vector4<T> {
	[r[0] - o, r[1] - o, r[2] + o + o, r[3] + o + o]
}

pub fn rect_ltrb<T: Float>(left: T, top: T, right: T, bottom: T) -> Vector4<T> {
	[ left, top, (right-left).max(zero()), (bottom-top).max(zero()) ]
}

pub fn rect_centered_on_origin<T: Float + FromPrimitive>(width: T, height: T) -> Vector4<T> {
	[ width * -half::<T>(), height * -half::<T>(), width, height ]
}

pub fn rect_intersect<T: Float>(a: Vector4<T>, b: Vector4<T>) -> Vector4<T> {
	rect_ltrb(
		rect_left(a).max(rect_left(b)),
		rect_top(a).max(rect_top(b)),
		rect_right(a).min(rect_right(b)),
		rect_bottom(a).min(rect_bottom(b))
	)
}

pub fn rect_constrain<T: Float>(input: Vector4<T>, container: Vector4<T>) -> Vector4<T> {
	// Keep the input within the container, avoiding resizing
	// Shrink the input only if it's larger than the container:
	let input = [ input[0], input[1], input[2].min(container[2]), input[3].min(container[3]) ];
	[
		input[0].max(rect_left(container)).min(rect_right(container) - input[2]),
		input[1].max(rect_top(container)).min(rect_bottom(container) - input[3]),
		input[2],
		input[3],
	]
}

pub fn rect_translate<T: Float>(rect: Vector4<T>, tr: Vector2<T>) -> Vector4<T> {
	[ rect[0] + tr[0], rect[1] + tr[1], rect[2], rect[3] ]
}

pub fn rect_nearest_perimeter_point<T: Float + FromPrimitive>(rect: Vector4<T>, p: Vector2<T>) -> Vector2<T> {
	if rect[2] <= zero() || rect[3] <= zero() {
		rect_topleft(rect)
	} else {
		let rel = vec2_sub(p, rect_center(rect));
		if rel[1].abs() / rel[0].abs() > rect[3] / rect[2] {
			// Top/bottom edge
			[
				p[0],
				if rel[1] > zero() { rect_bottom(rect) } else { rect_top(rect) }
			]
		} else {
			// Left/right edge
			[
				if rel[0] > zero() { rect_right(rect) } else { rect_left(rect) },
				p[1]
			]
		}
	}
}

pub fn vec2_rand_from_centered_unit_square() -> Vector2<f32> {
	[ thread_rng().next_f32() - 0.5, thread_rng().next_f32() - 0.5 ]
}

pub fn vec2_clamp_len<T: Float + Sqrt>(vec: Vector2<T>, limit: T) -> Vector2<T> {
    let len = vec2_len(vec);
    if len < limit { vec } else { vec2_scale(vec, limit / len) }
}
