use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::chord::{Chord, SegmentChord};
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::quad::QuadSegment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy)]
pub enum Segment<I: IntNumber> {
    Line(LineSegment<I>),
    Quad(QuadSegment<I>),
    Cubic(CubicSegment<I>),
}

impl<I: IntNumber> Default for Segment<I> {
    fn default() -> Self {
        Self::Line(LineSegment {
            control_points: [IntPoint::ZERO; 2],
        })
    }
}

impl<I: IntNumber> Segment<I> {
    #[inline]
    pub fn convex_hull(&self) -> StackVec<IntPoint<I>, 4> {
        match self {
            Segment::Line(line) => StackVec::with_slice_as_convex(&line.control_points),
            Segment::Quad(quad) => StackVec::with_slice_as_convex(&quad.control_points),
            Segment::Cubic(cubic) => StackVec::with_slice_as_convex(&cubic.control_points),
        }
    }
}

impl<I: IntNumber> Chord<I> for Segment<I> {
    #[inline]
    fn chord(&self) -> SegmentChord<I> {
        match self {
            Segment::Line(line) => line.chord(),
            Segment::Quad(quad) => quad.chord(),
            Segment::Cubic(cubic) => cubic.chord(),
        }
    }
}
