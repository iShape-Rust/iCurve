use crate::int::bezier::spline::IntCADSpline;
use crate::int::bezier::split::LineDivider;
use crate::int::math::normalize::Normalize16;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntCubeSpline {
    pub a: IntPoint,
    pub am: IntPoint,
    pub bm: IntPoint,
    pub b: IntPoint,
}
impl IntCADSpline for IntCubeSpline {
    #[inline]
    fn start(&self) -> IntPoint {
        self.a
    }
    #[inline]
    fn start_dir(&self) -> IntPoint {
        (self.am - self.a).normalized_16bit()
    }
    #[inline]
    fn end_dir(&self) -> IntPoint {
        (self.b - self.bm).normalized_16bit()
    }
    #[inline]
    fn end(&self) -> IntPoint {
        self.b
    }

    #[inline]
    fn split_at(&self, step: usize, split_factor: u32) -> IntPoint {
        let l0 = LineDivider::new(self.a, self.am);
        let l1 = LineDivider::new(self.am, self.bm);
        let l2 = LineDivider::new(self.bm, self.b);

        let p0 = l0.split_at(step, split_factor);
        let p1 = l1.split_at(step, split_factor);
        let p2 = l2.split_at(step, split_factor);

        let p10 = LineDivider::new(p0, p1).split_at(step, split_factor);
        let p11 = LineDivider::new(p1, p2).split_at(step, split_factor);

        LineDivider::new(p10, p11).split_at(step, split_factor)
    }

    #[inline]
    fn boundary(&self) -> IntRect {
        let mut boundary = IntRect::empty();
        boundary.add_point(&self.a);
        boundary.add_point(&self.am);
        boundary.add_point(&self.bm);
        boundary.add_point(&self.b);
        boundary
    }
}
