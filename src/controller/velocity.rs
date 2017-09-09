use bus::*;
use vecmath::*;
use config::Config;

pub struct RateLimitedVelocity {
    vec: Vector3<f64>
}

impl RateLimitedVelocity {
    pub fn new() -> RateLimitedVelocity {
        RateLimitedVelocity {
            vec: [0.0, 0.0, 0.0]
        }
    }

    pub fn tick(self: &mut RateLimitedVelocity, config: &Config, target: Vector3<f64>) {
        let dt = 1.0 / (TICK_HZ as f64);
        let limit_per_tick = config.params.accel_limit_m_per_sec2 * dt;
        let diff = vec3_sub(target, self.vec);
        let len = vec3_len(diff);
        let clipped = if len > limit_per_tick {
            vec3_scale(diff, limit_per_tick / len)
        } else {
            diff
        };
        self.vec = vec3_add(self.vec, clipped);
    }

    pub fn get(self: &RateLimitedVelocity) -> Vector3<f64> {
        self.vec
    }
}
