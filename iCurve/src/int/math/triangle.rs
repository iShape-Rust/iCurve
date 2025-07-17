use crate::int::math::point::IntPoint;

pub(crate) struct Triangle;

impl Triangle {
    #[inline(always)]
    pub(crate) fn area_two(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> i64 {
        (p1 - p0).cross_product(&(p1 - p2))
    }

    #[inline(always)]
    pub(crate) fn clock_direction(p0: IntPoint, p1: IntPoint, p2: IntPoint) -> i64 {
        Self::area_two(p0, p1, p2).signum()
    }
}