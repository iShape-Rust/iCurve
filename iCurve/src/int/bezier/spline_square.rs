use crate::int::bezier::spline::{IntBezierSplineMath, SplitPosition};
use crate::int::bezier::position::LineDivider;
use crate::int::math::normalize::VectorNormalization16;
use crate::int::math::point::IntPoint;

#[derive(Debug, Clone)]
pub struct IntSquareSpline {
    pub(crate) anchors: [IntPoint; 3]
}

impl IntBezierSplineMath for IntSquareSpline {
    #[inline]
    fn start_dir(&self) -> IntPoint {
        (self.anchors[1] - self.anchors[0]).normalized_16bit()
    }
    #[inline]
    fn end_dir(&self) -> IntPoint {
        (self.anchors[2] - self.anchors[1]).normalized_16bit()
    }

    #[inline]
    fn point_at(&self, position: &SplitPosition) -> IntPoint {
        let a = self.anchors[0];
        let m = self.anchors[1];
        let b = self.anchors[2];

        let ma = LineDivider::new(a, m).point_at(position);
        let mb = LineDivider::new(m, b).point_at(position);

        LineDivider::new(ma, mb).point_at(position)
    }

    #[inline]
    fn bisect(&self) -> (Self, Self) {
        let a = self.anchors[0];
        let m = self.anchors[1];
        let b = self.anchors[2];
        
        let ma = a.mid(&m);
        let mb = m.mid(&b);
        
        let mm = ma.mid(&mb);
        
        (Self { anchors: [a, ma, mm] }, Self { anchors: [mm, mb, b] })
    }

    #[inline]
    fn split(&self, position: &SplitPosition) -> (Self, Self) {
        let a = self.anchors[0];
        let m = self.anchors[1];
        let b = self.anchors[2];

        let ma = LineDivider::new(a, m).point_at(position);
        let mb = LineDivider::new(m, b).point_at(position);

        let mm = LineDivider::new(m, b).point_at(position);
        
        (Self { anchors: [a, ma, mm] }, Self { anchors: [mm, mb, b] })
    }
}