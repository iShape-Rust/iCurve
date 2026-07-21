use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

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

impl<I: IntNumber> Segment<I> {
    pub(crate) fn split_at_point(&self, t: SegmentParam<I>, point: IntPoint<I>) -> [Self; 2] {
        match self {
            Segment::Line(line) => {
                let [mut left, mut right] = line.split_at(t);
                left.control_points[1] = point;
                right.control_points[0] = point;
                [Segment::Line(left), Segment::Line(right)]
            }
            Segment::Quad(quad) => {
                let [mut left, mut right] = quad.split_at(t);
                left.control_points[2] = point;
                right.control_points[0] = point;
                [Segment::Quad(left), Segment::Quad(right)]
            }
            Segment::Cubic(cubic) => {
                let [mut left, mut right] = cubic.split_at(t);
                left.control_points[3] = point;
                right.control_points[0] = point;
                [Segment::Cubic(left), Segment::Cubic(right)]
            }
        }
    }
}

#[cfg(test)]
mod segment_tests {
    use super::*;

    #[test]
    fn split_at_point_uses_requested_shared_point() {
        let segment = Segment::Quad(QuadSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(5, 8), IntPoint::new(10, 0)],
        });
        let point = IntPoint::new(5, 5);

        let [left, right] = segment.split_at_point(SegmentParam::half(), point);

        match (left, right) {
            (Segment::Quad(left), Segment::Quad(right)) => {
                assert_eq!(left.control_points[2], point);
                assert_eq!(right.control_points[0], point);
            }
            _ => panic!("expected quadratic segments"),
        }
    }
}
