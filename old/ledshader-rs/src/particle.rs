
use palette::{Rgb, Hsl, RgbHue, IntoColor};
use kdtree::kdtree::KdtreePointTrait;
use cgmath::{Point3, EuclideanSpace};


struct LedInfo {
    pub loc: Point3<f64>,
    pub visibility: u64,
}


#[derive(Copy, PartialEq, Clone, Debug)]
struct Particle {
    pub loc: Point3<f64>,
    pub color: Rgb,
    pub radius: f64,
    pub intensity: f64,
    pub visibility: u64
}


impl KdtreePointTrait for Particle {
    #[inline]
    fn dims(&self) -> &[f64] {
        let r: &[f64; 3] = self.loc.as_ref();
        &r[..]
    }
}


impl Particle {

    pub fn render(&self, dist_squared: f64) -> Rgb {
        color * intensity * kernel_q2(dist_squared)
    }

    /// Kernel function; determines particle shape
    /// Poly6 kernel, Müller, Charypar, & Gross (2003)
    /// q normalized in range [0, 1].
    /// Has compact support; kernel forced to zero outside this range.
    pub fn kernel(q: f64) -> f64 {
        let a = 1.0 - q * q;
        a * a * a
    }

    /// Variant of kernel function called with q^2
    /// to avoid taking unnecessary square roots.
    pub fn kernel_q2(q_squared: f64) -> f64 {
        let a = 1.0 - q_squared;
        a * a * a
    }

    /// First derivative of kernel()
    pub fn kernel_derivative(q: f64) -> f64 {
        let a = 1.0 - q * q;
        -6.0 * q * a * a
    }
}

