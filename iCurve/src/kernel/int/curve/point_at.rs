use crate::kernel::int::curve::arc::ArcSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub trait PointAt<I: IntNumber> {
    fn point_at(&self, param: SegmentParam<I>) -> IntPoint<I>;
}

impl<I: IntNumber> PointAt<I> for ArcSegment<I> {
    #[inline(always)]
    fn point_at(&self, param: SegmentParam<I>) -> IntPoint<I> {
        ArcSegment::point_at(self, param)
    }
}

impl<I: IntNumber> PointAt<I> for Segment<I> {
    #[inline(always)]
    fn point_at(&self, param: SegmentParam<I>) -> IntPoint<I> {
        match self {
            Segment::Line(line) => line.control_points.point_at(param),
            Segment::Quad(quad) => quad.control_points.point_at(param),
            Segment::Cubic(cubic) => cubic.control_points.point_at(param),
            Segment::Arc(arc) => arc.point_at(param),
        }
    }
}

impl<I: IntNumber> PointAt<I> for [IntPoint<I>; 2] {
    #[inline(always)]
    fn point_at(&self, t: SegmentParam<I>) -> IntPoint<I> {
        let [p0, p1] = *self;
        p0 + t.scale_vector_to_point(p1 - p0)
    }
}

impl<I: IntNumber> PointAt<I> for [IntPoint<I>; 3] {
    #[inline(always)]
    fn point_at(&self, t: SegmentParam<I>) -> IntPoint<I> {
        let [p0, p1, p2] = *self;
        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        [p01, p12].point_at(t)
    }
}

impl<I: IntNumber> PointAt<I> for [IntPoint<I>; 4] {
    #[inline(always)]
    fn point_at(&self, t: SegmentParam<I>) -> IntPoint<I> {
        let [p0, p1, p2, p3] = *self;
        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p23 = [p2, p3].point_at(t);

        let p012 = [p01, p12].point_at(t);
        let p123 = [p12, p23].point_at(t);

        [p012, p123].point_at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::PointAt;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
    use crate::kernel::int::curve::param::SegmentParam;
    use crate::kernel::int::curve::segment::Segment;
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn line_point_at_uses_rounded_scale() {
        let line = [IntPoint::new(0, 0), IntPoint::new(1, -1)];

        assert_eq!(line.point_at(SegmentParam::half()), IntPoint::new(1, -1));
    }

    #[test]
    fn quad_point_at_uses_rounded_de_casteljau_steps() {
        let quad = [IntPoint::new(0, 0), IntPoint::new(2, 0), IntPoint::new(2, 2)];

        assert_eq!(quad.point_at(SegmentParam::half()), IntPoint::new(2, 1));
    }

    #[test]
    fn cubic_point_at_uses_rounded_de_casteljau_steps() {
        let cubic = [
            IntPoint::new(0, 0),
            IntPoint::new(4, 0),
            IntPoint::new(4, 4),
            IntPoint::new(8, 4),
        ];

        assert_eq!(cubic.point_at(SegmentParam::half()), IntPoint::new(4, 2));
    }

    #[test]
    fn segment_dispatches_arc_point_at() {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let arc = ArcSegment {
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
        };
        let segment = Segment::Arc(arc);

        assert_eq!(
            segment.point_at(SegmentParam::half()),
            arc.point_at(SegmentParam::half())
        );
        assert_eq!(segment.point_at(SegmentParam::half()), IntPoint::new(71, 71));
    }
}
