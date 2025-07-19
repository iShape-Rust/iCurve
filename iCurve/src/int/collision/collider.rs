use crate::data::four_vec::FourVec;
use crate::int::collision::colliding::{Colliding, CollidingResult};
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

pub struct FourCollider {
    pub(crate) boundary: IntRect,
    pub(crate) convex: FourVec<IntPoint>,
}

impl Colliding for FourCollider {
    #[inline]
    fn collide(&self, other: &Self) -> CollidingResult {
        if !self.boundary.is_intersect_border_include(&other.boundary) {
            return CollidingResult::None;
        }
        self.convex.collide(&other.convex)
    }
}