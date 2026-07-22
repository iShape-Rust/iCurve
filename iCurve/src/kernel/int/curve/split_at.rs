use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

pub trait SplitAt<I: IntNumber> {
    type Output;
    fn split_at(&self, t: SegmentParam<I>) -> Self::Output;
    fn split_at_left(&self, t: SegmentParam<I>) -> Self;
    fn split_at_right(&self, t: SegmentParam<I>) -> Self;
}

#[inline]
pub(crate) fn segment_range<I, S>(segment: &S, start: SegmentParam<I>, end: SegmentParam<I>) -> S
where
    I: IntNumber,
    S: SplitAt<I, Output = [S; 2]> + Copy,
{
    if start.value() == I::Wide::ZERO && end.value() == SegmentParam::<I>::DENOMINATOR {
        return *segment;
    }

    if start.value() == I::Wide::ZERO {
        return segment.split_at_left(end);
    }

    let right = segment.split_at_right(start);
    if end.value() == SegmentParam::<I>::DENOMINATOR {
        return right;
    }

    let numerator = end.value() - start.value();
    let denominator = SegmentParam::<I>::DENOMINATOR - start.value();
    let local = SegmentParam::from_int(I::from_wide(numerator), I::from_wide(denominator));

    right.split_at_left(local)
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

    pub(crate) fn subsegment(
        &self,
        start_param: SegmentParam<I>,
        start_point: IntPoint<I>,
        end_param: SegmentParam<I>,
        end_point: IntPoint<I>,
    ) -> Option<Self> {
        let start_value = start_param.value();
        let end_value = end_param.value();
        if start_value == end_value {
            return None;
        }

        let reverse = start_value > end_value;
        let (range_start, range_end) = if reverse {
            (end_param, start_param)
        } else {
            (start_param, end_param)
        };

        let mut result = match self {
            Segment::Line(line) => Segment::Line(segment_range(line, range_start, range_end)),
            Segment::Quad(quad) => Segment::Quad(segment_range(quad, range_start, range_end)),
            Segment::Cubic(cubic) => Segment::Cubic(segment_range(cubic, range_start, range_end)),
        };

        if reverse {
            match &mut result {
                Segment::Line(line) => line.control_points.reverse(),
                Segment::Quad(quad) => quad.control_points.reverse(),
                Segment::Cubic(cubic) => cubic.control_points.reverse(),
            }
        }

        match &mut result {
            Segment::Line(line) => {
                line.control_points[0] = start_point;
                line.control_points[1] = end_point;
            }
            Segment::Quad(quad) => {
                quad.control_points[0] = start_point;
                quad.control_points[2] = end_point;
            }
            Segment::Cubic(cubic) => {
                cubic.control_points[0] = start_point;
                cubic.control_points[3] = end_point;
            }
        }

        Some(result)
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

    #[test]
    fn extracts_and_reverses_cubic_subsegment() {
        let segment = Segment::Cubic(CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 8),
                IntPoint::new(8, 8),
                IntPoint::new(8, 0),
            ],
        });
        let middle = IntPoint::new(4, 6);

        let forward = segment
            .subsegment(
                SegmentParam::new(0),
                IntPoint::new(0, 0),
                SegmentParam::half(),
                middle,
            )
            .unwrap();
        let reverse = segment
            .subsegment(
                SegmentParam::half(),
                middle,
                SegmentParam::new(0),
                IntPoint::new(0, 0),
            )
            .unwrap();

        match (forward, reverse) {
            (Segment::Cubic(forward), Segment::Cubic(reverse)) => {
                assert_eq!(
                    forward.control_points,
                    [
                        IntPoint::new(0, 0),
                        IntPoint::new(0, 4),
                        IntPoint::new(2, 6),
                        middle,
                    ]
                );
                assert_eq!(
                    reverse.control_points,
                    [
                        middle,
                        IntPoint::new(2, 6),
                        IntPoint::new(0, 4),
                        IntPoint::new(0, 0),
                    ]
                );
            }
            _ => panic!("expected cubic segments"),
        }
    }
}
