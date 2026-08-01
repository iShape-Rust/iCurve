use crate::int::CurveInt;
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_shape::int::IntPoint;

pub(crate) trait Bisect<I: CurveInt> {
    fn bisect(&self, start: IntPoint<I>, end: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2]
    where
        Self: Sized;
}

impl<I: CurveInt> Bisect<I> for [IntPoint<I>; 2] {
    fn bisect(&self, a: IntPoint<I>, b: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2] {
        let m = self.point_at(t);
        [[a, m], [m, b]]
    }
}

impl<I: CurveInt> Bisect<I> for [IntPoint<I>; 3] {
    fn bisect(&self, a: IntPoint<I>, b: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2] {
        let [p0, p1, p2] = *self;
        let m01 = [p0, p1].point_at(t);
        let m12 = [p1, p2].point_at(t);
        let m = [m01, m12].point_at(t);

        [[a, m01, m], [m, m12, b]]
    }
}

impl<I: CurveInt> Bisect<I> for [IntPoint<I>; 4] {
    fn bisect(&self, a: IntPoint<I>, b: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2] {
        let [p0, p1, p2, p3] = *self;
        let m01 = [p0, p1].point_at(t);
        let m12 = [p1, p2].point_at(t);
        let m23 = [p2, p3].point_at(t);
        let m012 = [m01, m12].point_at(t);
        let m123 = [m12, m23].point_at(t);
        let m = [m012, m123].point_at(t);

        [[a, m01, m012, m], [m, m123, m23, b]]
    }
}

impl<I: CurveInt> Segment<I> {
    pub(crate) fn bisect(&self, a: IntPoint<I>, b: IntPoint<I>, t: SegmentParam<I>) -> [Option<Self>; 2] {
        match self {
            Segment::Line(line) => {
                let [l0, l1] = line.control_points.bisect(a, b, t);
                let n0 = LineSegment { control_points: l0 }.try_segment();
                let n1 = LineSegment { control_points: l1 }.try_segment();

                [n0, n1]
            }
            Segment::Quad(quad) => {
                let [q0, q1] = quad.control_points.bisect(a, b, t);
                let n0 = QuadSegment { control_points: q0 }.try_segment();
                let n1 = QuadSegment { control_points: q1 }.try_segment();

                [n0, n1]
            }
            Segment::Cubic(cubic) => {
                let [c0, c1] = cubic.control_points.bisect(a, b, t);
                let n0 = CubicSegment { control_points: c0 }.try_cubic_without_self_intersection();
                let n1 = CubicSegment { control_points: c1 }.try_cubic_without_self_intersection();

                [n0, n1]
            }
            Segment::Arc(arc) => {
                let [mut left, mut right] = arc.rational_split(t);
                left.control_points[0] = a;
                right.control_points[2] = b;

                let split = left.control_points[2];
                let n0 = (a != split).then_some(Segment::Arc(left));
                let n1 = (split != b).then_some(Segment::Arc(right));

                [n0, n1]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bisect;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
    use crate::kernel::int::curve::param::SegmentParam;
    use crate::kernel::int::curve::segment::Segment;
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn line_00() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(10, 20);

        let [s0, s1] = [p0, p1].bisect(p0, p1, SegmentParam::half());

        let m = IntPoint::new(5, 10);

        assert_eq!(s0, [p0, m]);
        assert_eq!(s1, [m, p1]);
    }

    #[test]
    fn cubic_00() {
        let p0 = IntPoint::new(2, 2);
        let p1 = IntPoint::new(2, 10);
        let p2 = IntPoint::new(8, 10);
        let p3 = IntPoint::new(8, 2);

        let [s0, s1] = [p0, p1, p2, p3].bisect(p0, p1, SegmentParam::half());

        let m = IntPoint::new(6, 8);

        assert_eq!(s0, [p0, IntPoint::new(2, 6), IntPoint::new(4, 8), m]);
        assert_eq!(s1, [m, IntPoint::new(7, 8), IntPoint::new(8, 6), p1]);
    }

    #[test]
    fn arc_bisection_uses_requested_outer_endpoints() {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let segment = Segment::Arc(ArcSegment {
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
        });
        let requested_start = IntPoint::new(99, 0);
        let requested_end = IntPoint::new(0, 99);

        let [Some(Segment::Arc(left)), Some(Segment::Arc(right))] =
            segment.bisect(requested_start, requested_end, SegmentParam::half())
        else {
            panic!("expected two arc segments");
        };

        assert_eq!(left.control_points[0], requested_start);
        assert_eq!(right.control_points[2], requested_end);
        assert_eq!(left.control_points[2], right.control_points[0]);
    }
}
