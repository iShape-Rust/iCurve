use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;
use crate::kernel::int::curve::chord::{Chord, SegmentChord};

#[derive(Debug, Clone, Copy, Default)]
pub struct QuadSegment<I: IntNumber> {
    pub control_points: [IntPoint<I>; 3],
}

impl<I: IntNumber> Chord<I> for QuadSegment<I> {
    #[inline]
    fn chord(&self) -> SegmentChord<I> {
        SegmentChord { a: self.control_points[0], b: self.control_points[2] }
    }
}