use crate::int::collision::spline::Spline;
use crate::int::math::rect::IntRect;

pub trait SplineCollider {
    fn boundary(&self) -> IntRect;
}

pub struct Collider {
    spline: Spline,
    boundary: IntRect
}

impl Collider {
    fn new(spline: Spline) -> Self {
        Self {
            boundary: spline.boundary(),
            spline,
        }
    }
}