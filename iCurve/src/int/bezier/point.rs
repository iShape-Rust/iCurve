use alloc::vec::Vec;
use crate::int::bezier::short::IntSplineShorts;
use crate::int::bezier::spline::IntBezierSplineApi;
use crate::int::math::point::IntPoint;

pub trait IntSplinePoints {
    fn approximate_points(&self, min_cos: u32, min_len: u32) -> Vec<IntPoint>;
}

impl<Spline: IntBezierSplineApi> IntSplinePoints for Spline {
    #[inline]
    fn approximate_points(&self, min_cos: u32, min_len: u32) -> Vec<IntPoint> {
        let shorts = self.approximate(min_cos, min_len);
        let mut points: Vec<_> = shorts.iter().map(|s| s.a).collect();
        points.push(shorts.last().unwrap().b);

        points
    }
}