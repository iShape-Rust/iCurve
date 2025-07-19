use crate::int::bezier::spline::IntCADSpline;
use crate::int::bezier::split::LineDivider;
use crate::int::math::normalize::VectorNormalization16;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntLineSpline {
    pub(super) anchors: [IntPoint; 2]
}

impl IntCADSpline for IntLineSpline {

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
        (self.anchors[1] - self.anchors[0]).normalized_16bit()
    }

    #[inline]
    fn end(&self) -> IntPoint {
        self.anchors[1]
    }

    #[inline]
    fn split_at(&self, step: usize, split_factor: u32) -> IntPoint {
        LineDivider::new(self.anchors[0], self.anchors[1]).split_at(step, split_factor)
    }

    #[inline]
    fn bisect(&self) -> (Self, Self) {
        let a = self.anchors[0];
        let b = self.anchors[1];
        let m = a.mid(&b);
        (Self { anchors: [a, m] }, Self { anchors: [m, b] })
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
