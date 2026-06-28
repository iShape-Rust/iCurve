use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use i_overlay::i_float::int::number::int::IntNumber;

pub trait SplitAt<I: IntNumber> {
    type Output;
    fn split_at(&self, t: SegmentParam<I>) -> Self::Output;
    fn split_at_left(&self, t: SegmentParam<I>) -> Self;
    fn split_at_right(&self, t: SegmentParam<I>) -> Self;
}

impl<I: IntNumber> SplitAt<I> for LineSegment<I> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: SegmentParam<I>) -> Self::Output {
        let m = self.control_points.point_at(t);
        let [p0, p1] = self.control_points;
        [
            LineSegment {
                control_points: [p0, m],
            },
            LineSegment {
                control_points: [m, p1],
            },
        ]
    }

    #[inline]
    fn split_at_left(&self, t: SegmentParam<I>) -> Self {
        let m = self.control_points.point_at(t);
        LineSegment {
            control_points: [self.control_points[0], m],
        }
    }

    #[inline]
    fn split_at_right(&self, t: SegmentParam<I>) -> Self {
        let m = self.control_points.point_at(t);
        LineSegment {
            control_points: [m, self.control_points[1]],
        }
    }
}

impl<I: IntNumber> SplitAt<I> for QuadSegment<I> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: SegmentParam<I>) -> Self::Output {
        let [p0, p1, p2] = self.control_points;

        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p012 = [p01, p12].point_at(t);

        [
            Self {
                control_points: [p0, p01, p012],
            },
            Self {
                control_points: [p012, p12, p2],
            },
        ]
    }

    #[inline]
    fn split_at_left(&self, t: SegmentParam<I>) -> Self {
        let [p0, p1, p2] = self.control_points;

        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p012 = [p01, p12].point_at(t);

        Self {
            control_points: [p0, p01, p012],
        }
    }

    #[inline]
    fn split_at_right(&self, t: SegmentParam<I>) -> Self {
        let [p0, p1, p2] = self.control_points;

        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p012 = [p01, p12].point_at(t);

        Self {
            control_points: [p012, p12, p2],
        }
    }
}

impl<I: IntNumber> SplitAt<I> for CubicSegment<I> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: SegmentParam<I>) -> Self::Output {
        let [p0, p1, p2, p3] = self.control_points;
        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p23 = [p2, p3].point_at(t);
        let p012 = [p01, p12].point_at(t);
        let p123 = [p12, p23].point_at(t);
        let p0123 = [p012, p123].point_at(t);

        [
            Self {
                control_points: [p0, p01, p012, p0123],
            },
            Self {
                control_points: [p0123, p123, p23, p3],
            },
        ]
    }

    #[inline]
    fn split_at_left(&self, t: SegmentParam<I>) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p23 = [p2, p3].point_at(t);
        let p012 = [p01, p12].point_at(t);
        let p123 = [p12, p23].point_at(t);
        let p0123 = [p012, p123].point_at(t);

        Self {
            control_points: [p0, p01, p012, p0123],
        }
    }

    #[inline]
    fn split_at_right(&self, t: SegmentParam<I>) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p23 = [p2, p3].point_at(t);
        let p012 = [p01, p12].point_at(t);
        let p123 = [p12, p23].point_at(t);
        let p0123 = [p012, p123].point_at(t);

        Self {
            control_points: [p0123, p123, p23, p3],
        }
    }
}
