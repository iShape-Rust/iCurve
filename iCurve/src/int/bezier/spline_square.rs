use crate::int::bezier::spline::IntCADSpline;
use crate::int::bezier::split::LineDivider;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntSquareSpline {
    pub a: IntPoint,
    pub m: IntPoint,
    pub b: IntPoint,
}

impl IntCADSpline for IntSquareSpline {
    #[inline]
    fn start(&self) -> IntPoint {
        self.a
    }
    #[inline]
    fn start_dir(&self) -> IntPoint {
        (self.m - self.a).normalized_10bit()
    }
    #[inline]
    fn end_dir(&self) -> IntPoint {
        (self.b - self.m).normalized_10bit()
    }
    #[inline]
    fn end(&self) -> IntPoint {
        self.b
    }

    #[inline]
    fn split_at(&self, step: usize, split_factor: u32) -> IntPoint {
        let l0 = LineDivider::new(self.a, self.m);
        let l1 = LineDivider::new(self.m, self.b);
        let p10 = l0.split_at(step, split_factor);
        let p11 = l1.split_at(step, split_factor);
        LineDivider::new(p10, p11).split_at(step, split_factor)
    }

    #[inline]
    fn boundary(&self) -> IntRect {
        let mut boundary = IntRect::empty();
        boundary.add_point(&self.a);
        boundary.add_point(&self.m);
        boundary.add_point(&self.b);
        boundary
    }
}
