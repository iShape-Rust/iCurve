use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub(crate) trait Bisect<I: IntNumber> {
    fn bisect(&self, start: IntPoint<I>, end: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2]
    where
        Self: Sized;
}

impl<I: IntNumber> Bisect<I> for [IntPoint<I>; 2] {
    fn bisect(&self, a: IntPoint<I>, b: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2] {
        let m = self.point_at(t);
        [[a, m], [m, b]]
    }
}

impl<I: IntNumber> Bisect<I> for [IntPoint<I>; 3] {
    fn bisect(&self, a: IntPoint<I>, b: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2] {
        let [p0, p1, p2] = *self;
        let m01 = [p0, p1].point_at(t);
        let m12 = [p1, p2].point_at(t);
        let m = [m01, m12].point_at(t);

        [[a, m01, m], [m, m12, b]]
    }
}

impl<I: IntNumber> Bisect<I> for [IntPoint<I>; 4] {
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

impl<I: IntNumber> Bisect<I> for Segment<I> {
    fn bisect(&self, a: IntPoint<I>, b: IntPoint<I>, t: SegmentParam<I>) -> [Self; 2] {
        match self {
            Segment::Line(line) => {
                let [l0, l1] = line.control_points.bisect(a, b, t);
                [
                    Segment::Line(LineSegment { control_points: l0 }),
                    Segment::Line(LineSegment { control_points: l1 }),
                ]
            }
            Segment::Quad(quad) => {
                let [q0, q1] = quad.control_points.bisect(a, b, t);
                [
                    Segment::Quad(QuadSegment { control_points: q0 }),
                    Segment::Quad(QuadSegment { control_points: q1 }),
                ]
            }
            Segment::Cubic(cubic) => {
                let [c0, c1] = cubic.control_points.bisect(a, b, t);
                [
                    Segment::Cubic(CubicSegment { control_points: c0 }),
                    Segment::Cubic(CubicSegment { control_points: c1 }),
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bisect;
    use crate::kernel::int::curve::param::SegmentParam;
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
}
