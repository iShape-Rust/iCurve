use crate::collections::stack_vec::StackVec;
use crate::int::CurveInt;
use crate::kernel::int::curve::arc::ArcSegment;
use crate::kernel::int::curve::chord::{Chord, SegmentChord};
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::quad::QuadSegment;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Segment<I: CurveInt> {
    Line(LineSegment<I>),
    Quad(QuadSegment<I>),
    Cubic(CubicSegment<I>),
    Arc(ArcSegment<I>),
}

impl<I: CurveInt> Default for Segment<I> {
    fn default() -> Self {
        Self::Line(LineSegment {
            control_points: [IntPoint::ZERO; 2],
        })
    }
}

impl<I: CurveInt> Segment<I> {
    #[inline]
    pub(crate) fn convex_hull(&self) -> StackVec<IntPoint<I>, 4> {
        match self {
            Segment::Line(line) => StackVec::with_slice_as_convex(&line.control_points),
            Segment::Quad(quad) => StackVec::with_slice_as_convex(&quad.control_points),
            Segment::Cubic(cubic) => StackVec::with_slice_as_convex(&cubic.control_points),
            Segment::Arc(arc) => StackVec::with_slice_as_convex(&arc.control_points),
        }
    }
}

impl<I: CurveInt> Chord<I> for Segment<I> {
    #[inline]
    fn chord(&self) -> SegmentChord<I> {
        match self {
            Segment::Line(line) => line.chord(),
            Segment::Quad(quad) => quad.chord(),
            Segment::Cubic(cubic) => cubic.chord(),
            Segment::Arc(arc) => SegmentChord {
                a: arc.control_points[0],
                b: arc.control_points[2],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcVector, EllipseFrame};
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;

    #[test]
    fn arc_chord_and_convex_hull_use_rational_controls() {
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

        let chord = segment.chord();
        assert_eq!(chord.a, arc.control_points[0]);
        assert_eq!(chord.b, arc.control_points[2]);

        let hull = segment.convex_hull();
        assert_eq!(hull.len(), 3);
        for point in arc.control_points {
            assert!(hull.as_slice().contains(&point));
        }
    }
}
