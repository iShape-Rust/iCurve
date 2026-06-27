use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
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
