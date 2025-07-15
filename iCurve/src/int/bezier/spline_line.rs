use crate::int::bezier::spline::IntCADSpline;
use crate::int::bezier::split::LineDivider;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntLineSpline {
    pub a: IntPoint,
    pub b: IntPoint,
}

impl IntCADSpline for IntLineSpline {
    #[inline]
    fn start(&self) -> IntPoint {
        self.a
    }

    #[inline]
    fn start_dir(&self) -> IntPoint {
        (self.b - self.a).normalized_10bit()
    }

    #[inline]
    fn end_dir(&self) -> IntPoint {
        (self.b - self.a).normalized_10bit()
    }

    #[inline]
    fn end(&self) -> IntPoint {
        self.b
    }

    #[inline]
    fn split_at(&self, step: usize, split_factor: u32) -> IntPoint {
        LineDivider::new(self.a, self.b).split_at(step, split_factor)
    }

    #[inline]
    fn boundary(&self) -> IntRect {
        let mut boundary = IntRect::empty();
        boundary.add_point(&self.a);
        boundary.add_point(&self.b);
        boundary
    }
}
