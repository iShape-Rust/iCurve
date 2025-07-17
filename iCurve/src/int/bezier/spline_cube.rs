use crate::int::bezier::spline::IntCADSpline;
use crate::int::bezier::split::LineDivider;
use crate::int::math::normalize::VectorNormalization16;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntCubeSpline {
    pub(super) anchors: [IntPoint; 4]
}
impl IntCADSpline for IntCubeSpline {
    #[inline]
    fn start(&self) -> IntPoint {
        self.anchors[0]
    }
    #[inline]
    fn start_dir(&self) -> IntPoint {
        (self.anchors[1] - self.anchors[0]).normalized_16bit()
    }
    #[inline]
    fn end_dir(&self) -> IntPoint {
        (self.anchors[3] - self.anchors[2]).normalized_16bit()
    }
    #[inline]
    fn end(&self) -> IntPoint {
        self.anchors[3]
    }

    #[inline]
    fn split_at(&self, step: usize, split_factor: u32) -> IntPoint {
        let l0 = LineDivider::new(self.anchors[0], self.anchors[1]);
        let l1 = LineDivider::new(self.anchors[1], self.anchors[2]);
        let l2 = LineDivider::new(self.anchors[2], self.anchors[3]);

        let p0 = l0.split_at(step, split_factor);
        let p1 = l1.split_at(step, split_factor);
        let p2 = l2.split_at(step, split_factor);

        let p10 = LineDivider::new(p0, p1).split_at(step, split_factor);
        let p11 = LineDivider::new(p1, p2).split_at(step, split_factor);

        LineDivider::new(p10, p11).split_at(step, split_factor)
    }

    #[inline]
    fn boundary(&self) -> IntRect {
        IntRect::with_points(&self.anchors)
    }

    #[inline]
    fn anchors(&self) -> &[IntPoint] {
        &self.anchors
    }
}
