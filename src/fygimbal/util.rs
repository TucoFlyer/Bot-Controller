use vecmath::*;

pub fn encoder_sub(a: i16, b: i16) -> i16 {
    const RANGE : i16 = 4096;
    let c = a.wrapping_sub(b) % RANGE;
    if c < -RANGE/2 {
        c + RANGE
    } else if c > RANGE/2 {
        c - RANGE
    } else {
        c
    }
}

pub fn vec2_encoder_sub(a: Vector2<i16>, b: Vector2<i16>) -> Vector2<i16> {
    [ encoder_sub(a[0], b[0]), encoder_sub(a[1], b[1]) ]
}
