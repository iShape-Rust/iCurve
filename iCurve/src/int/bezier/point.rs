use crate::int::bezier::short::Solver;
use crate::int::bezier::spline::IntCADSpline;
use crate::int::math::point::IntPoint;

pub trait IntSplinePoints {
    fn approximate_points(&self, min_cos: u32, min_len: u32) -> Vec<IntPoint>;
}

impl<Spline: IntCADSpline> IntSplinePoints for Spline {
    #[inline]
    fn approximate_points(&self, min_cos: u32, min_len: u32) -> Vec<IntPoint> {
        let shorts = Solver::approximate(self, min_cos, min_len);
        let mut points: Vec<_> = shorts.iter().map(|s| s.a).collect();
        points.push(shorts.last().unwrap().b);

        points
    }
}
