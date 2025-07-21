use crate::int::bezier::spline::{IntBezierSplineApi, SplitPosition};
use crate::int::bezier::position::LineDivider;
use crate::int::math::normalize::VectorNormalization16;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntCubeSpline {
    pub anchors: [IntPoint; 4],
}
impl IntBezierSplineApi for IntCubeSpline {
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
    fn point_at(&self, position: &SplitPosition) -> IntPoint {
        let l0 = LineDivider::new(self.anchors[0], self.anchors[1]);
        let l1 = LineDivider::new(self.anchors[1], self.anchors[2]);
        let l2 = LineDivider::new(self.anchors[2], self.anchors[3]);

        let p0 = l0.point_at(position);
        let p1 = l1.point_at(position);
        let p2 = l2.point_at(position);

        let p10 = LineDivider::new(p0, p1).point_at(position);
        let p11 = LineDivider::new(p1, p2).point_at(position);

        LineDivider::new(p10, p11).point_at(position)
    }

    #[inline]
    fn bisect(&self) -> (Self, Self) {
        let a = self.anchors[0];
        let ma = self.anchors[1];
        let mb = self.anchors[2];
        let b = self.anchors[3];

        let m0 = a.mid(&ma);
        let m1 = ma.mid(&mb);
        let m2 = mb.mid(&b);

        let mm0 = m0.mid(&m1);
        let mm1 = m1.mid(&m2);

        let mmm = mm0.mid(&mm1);

        let anchors_0 = [a, m0, mm0, mmm];
        let anchors_1 = [mmm, mm1, m2, b];

        (Self { anchors: anchors_0 }, Self { anchors: anchors_1 })
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
