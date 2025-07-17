use crate::int::bezier::spline::IntCADSpline;
use crate::int::bezier::split::LineDivider;
use crate::int::math::normalize::VectorNormalization16;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntSquareSpline {
    pub(super) anchors: [IntPoint; 3]
}

impl IntCADSpline for IntSquareSpline {
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
        (self.anchors[2] - self.anchors[1]).normalized_16bit()
    }
    #[inline]
    fn end(&self) -> IntPoint {
        self.anchors[2]
    }

    #[inline]
    fn split_at(&self, step: usize, split_factor: u32) -> IntPoint {
        let l0 = LineDivider::new(self.anchors[0], self.anchors[1]);
        let l1 = LineDivider::new(self.anchors[1], self.anchors[2]);
        let p10 = l0.split_at(step, split_factor);
        let p11 = l1.split_at(step, split_factor);
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
