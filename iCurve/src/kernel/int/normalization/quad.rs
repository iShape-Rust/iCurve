use crate::collections::stack_vec::StackVec;
use crate::int::CurveInt;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::normalization::monotone::decomposition::roots_to_segments;
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::vector::IntVector;
use i_overlay::i_float::triangle::Triangle;

impl<I: CurveInt> QuadSegment<I> {
    pub(crate) fn split_at_cusp(&self) -> StackVec<Self, 2> {
        let mut roots = StackVec::<SegmentParam<I>, 1>::new();
        roots.push_some(self.cusp_param());
        roots_to_segments(self, roots)
    }

    fn cusp_param(&self) -> Option<SegmentParam<I>> {
        let [p0, p1, p2] = self.control_points;
        let tangent = IntVector::<I>::new(p1.x.to_wide() - p0.x.to_wide(), p1.y.to_wide() - p0.y.to_wide());
        let delta = IntVector::<I>::new(
            p2.x.to_wide() - I::Wide::TWO * p1.x.to_wide() + p0.x.to_wide(),
            p2.y.to_wide() - I::Wide::TWO * p1.y.to_wide() + p0.y.to_wide(),
        );

        if delta.x == I::Wide::ZERO && delta.y == I::Wide::ZERO {
            return None;
        }
        if tangent.cross_product(delta) != I::Wide::ZERO {
            return None;
        }

        let t = if delta.x.unsigned_abs() >= delta.y.unsigned_abs() {
            FixedScale::<I>::div_to_scaled_round(-tangent.x, delta.x)
        } else {
            FixedScale::<I>::div_to_scaled_round(-tangent.y, delta.y)
        };

        if t > I::Wide::ZERO && t < SegmentParam::<I>::DENOMINATOR {
            Some(SegmentParam::new(I::from_wide(t)))
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn try_segment(self) -> Option<Segment<I>> {
        let [p0, p1, p2] = self.control_points;

        // Closed quadratic normalizes to an out-and-back spike.
        if p0 == p2 {
            None
        } else if Triangle::is_line(p0, p1, p2) {
            // Collinear quadratic contributes the same edge as its chord.
            LineSegment {
                control_points: [p0, p2],
            }
            .try_segment()
        } else {
            Some(Segment::Quad(self))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn drops_closed_quad_spike() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 0);
        let quad = QuadSegment {
            control_points: [p0, p1, p0],
        };

        let segment = quad.try_segment();

        assert!(segment.is_none());
    }

    #[test]
    fn reduces_collinear_quad_to_line() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 0);
        let p2 = IntPoint::new(2, 0);
        let quad = QuadSegment {
            control_points: [p0, p1, p2],
        };

        let segment = quad.try_segment();

        match segment {
            Some(Segment::Line(segment)) => assert_eq!(segment.control_points, [p0, p2]),
            _ => panic!("expected line segment"),
        }
    }

    #[test]
    fn keeps_non_degenerate_quad() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 1);
        let p2 = IntPoint::new(2, 0);
        let quad = QuadSegment {
            control_points: [p0, p1, p2],
        };

        let segment = quad.try_segment();

        match segment {
            Some(Segment::Quad(segment)) => assert_eq!(segment.control_points, [p0, p1, p2]),
            _ => panic!("expected quad segment"),
        }
    }

    #[test]
    fn splits_quad_at_cusp() {
        let quad = QuadSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(4, 0), IntPoint::new(2, 0)],
        };

        let segments = quad.split_at_cusp();
        let segments = segments.as_slice();

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].control_points[0], quad.control_points[0]);
        assert_eq!(segments[1].control_points[2], quad.control_points[2]);
        assert_eq!(segments[0].control_points[2], segments[1].control_points[0]);
    }
}
