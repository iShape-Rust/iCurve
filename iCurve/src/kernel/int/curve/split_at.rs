use crate::kernel::int::curve::arc::ArcSegment;
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

pub(crate) trait SplitAt<I: IntNumber> {
    type Output;
    fn split_at(&self, t: SegmentParam<I>) -> Self::Output;
    fn split_at_left(&self, t: SegmentParam<I>) -> Self;
    fn split_at_right(&self, t: SegmentParam<I>) -> Self;
}

pub(crate) trait SetSegmentEndpoints<I: IntNumber> {
    fn set_endpoints(&mut self, start: IntPoint<I>, end: IntPoint<I>);
}

#[inline]
pub(crate) fn segment_range<I, S>(
    segment: &S,
    start_param: SegmentParam<I>,
    start_point: IntPoint<I>,
    end_param: SegmentParam<I>,
    end_point: IntPoint<I>,
) -> S
where
    I: IntNumber,
    S: SplitAt<I, Output = [S; 2]> + SetSegmentEndpoints<I> + Copy,
{
    let mut result =
        if start_param.value() == I::Wide::ZERO && end_param.value() == SegmentParam::<I>::DENOMINATOR {
            *segment
        } else if start_param.value() == I::Wide::ZERO {
            segment.split_at_left(end_param)
        } else {
            let right = segment.split_at_right(start_param);
            if end_param.value() == SegmentParam::<I>::DENOMINATOR {
                right
            } else {
                let numerator = end_param.value() - start_param.value();
                let denominator = SegmentParam::<I>::DENOMINATOR - start_param.value();
                let local = SegmentParam::from_int(I::from_wide(numerator), I::from_wide(denominator));
                right.split_at_left(local)
            }
        };

    result.set_endpoints(start_point, end_point);
    result
}

impl<I: IntNumber> SetSegmentEndpoints<I> for LineSegment<I> {
    #[inline]
    fn set_endpoints(&mut self, start: IntPoint<I>, end: IntPoint<I>) {
        self.control_points[0] = start;
        self.control_points[1] = end;
    }
}

impl<I: IntNumber> SetSegmentEndpoints<I> for QuadSegment<I> {
    #[inline]
    fn set_endpoints(&mut self, start: IntPoint<I>, end: IntPoint<I>) {
        self.control_points[0] = start;
        self.control_points[2] = end;
    }
}

impl<I: IntNumber> SetSegmentEndpoints<I> for CubicSegment<I> {
    #[inline]
    fn set_endpoints(&mut self, start: IntPoint<I>, end: IntPoint<I>) {
        self.control_points[0] = start;
        self.control_points[3] = end;
    }
}

impl<I: IntNumber> SetSegmentEndpoints<I> for ArcSegment<I> {
    #[inline]
    fn set_endpoints(&mut self, start: IntPoint<I>, end: IntPoint<I>) {
        self.control_points[0] = start;
        self.control_points[2] = end;
    }
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

impl<I: IntNumber> SplitAt<I> for ArcSegment<I> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: SegmentParam<I>) -> Self::Output {
        self.rational_split(t)
    }

    #[inline]
    fn split_at_left(&self, t: SegmentParam<I>) -> Self {
        self.rational_split(t)[0]
    }

    #[inline]
    fn split_at_right(&self, t: SegmentParam<I>) -> Self {
        self.rational_split(t)[1]
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
            Segment::Arc(arc) => {
                let [mut left, mut right] = arc.split_at(t);
                left.control_points[2] = point;
                right.control_points[0] = point;
                [Segment::Arc(left), Segment::Arc(right)]
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
        let (range_start_param, range_start_point, range_end_param, range_end_point) = if reverse {
            (end_param, end_point, start_param, start_point)
        } else {
            (start_param, start_point, end_param, end_point)
        };

        let mut result = match self {
            Segment::Line(line) => Segment::Line(segment_range(
                line,
                range_start_param,
                range_start_point,
                range_end_param,
                range_end_point,
            )),
            Segment::Quad(quad) => Segment::Quad(segment_range(
                quad,
                range_start_param,
                range_start_point,
                range_end_param,
                range_end_point,
            )),
            Segment::Cubic(cubic) => Segment::Cubic(segment_range(
                cubic,
                range_start_param,
                range_start_point,
                range_end_param,
                range_end_point,
            )),
            Segment::Arc(arc) => Segment::Arc(segment_range(
                arc,
                range_start_param,
                range_start_point,
                range_end_param,
                range_end_point,
            )),
        };

        if reverse {
            match &mut result {
                Segment::Line(line) => line.control_points.reverse(),
                Segment::Quad(quad) => quad.control_points.reverse(),
                Segment::Cubic(cubic) => cubic.control_points.reverse(),
                Segment::Arc(arc) => arc.reverse(),
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod segment_tests {
    use super::*;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;

    fn quarter_circle() -> ArcSegment<i32> {
        let one = FixedScale::<i32>::DENOMINATOR as i32;

        ArcSegment {
            ellipse: EllipseFrame {
                center: IntPoint::new(0, 0),
                axis_x: ArcVector { x: 100, y: 0 },
                axis_y: ArcVector { x: 0, y: 100 },
            },
            control_points: [
                IntPoint::new(100, 0),
                IntPoint::new(100, 100),
                IntPoint::new(0, 100),
            ],
            weights: [one, 759_250_125, one],
            start_phase: ArcPhase { cos: one, sin: 0 },
            end_phase: ArcPhase { cos: 0, sin: one },
            direction: ArcDirection::CounterClockwise,
        }
    }

    #[test]
    fn segment_range_uses_requested_endpoints() {
        let segment = QuadSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(5, 8), IntPoint::new(10, 0)],
        };
        let start = IntPoint::new(3, 4);
        let end = IntPoint::new(7, 4);

        let range = segment_range(
            &segment,
            SegmentParam::from_int(1, 4),
            start,
            SegmentParam::from_int(3, 4),
            end,
        );

        assert_eq!(range.control_points[0], start);
        assert_eq!(range.control_points[2], end);
    }

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
    fn arc_split_at_point_uses_requested_shared_point() {
        let segment = Segment::Arc(quarter_circle());
        let point = IntPoint::new(70, 72);

        let [left, right] = segment.split_at_point(SegmentParam::half(), point);

        let (Segment::Arc(left), Segment::Arc(right)) = (left, right) else {
            panic!("expected arc segments");
        };
        assert_eq!(left.control_points[2], point);
        assert_eq!(right.control_points[0], point);
        assert_eq!(left.end_phase, right.start_phase);
    }

    #[test]
    fn extracts_and_reverses_arc_subsegment() {
        let arc = quarter_circle();
        let segment = Segment::Arc(arc);
        let middle = arc.point_at(SegmentParam::half());

        let reverse = segment
            .subsegment(
                SegmentParam::half(),
                middle,
                SegmentParam::new(0),
                arc.control_points[0],
            )
            .unwrap();

        let Segment::Arc(reverse) = reverse else {
            panic!("expected arc segment");
        };
        assert_eq!(reverse.control_points[0], middle);
        assert_eq!(reverse.control_points[2], arc.control_points[0]);
        assert_eq!(reverse.direction, ArcDirection::Clockwise);
        assert_eq!(reverse.start_phase.cos, 759_250_125);
        assert_eq!(reverse.start_phase.sin, 759_250_125);
        assert_eq!(reverse.end_phase, arc.start_phase);
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
