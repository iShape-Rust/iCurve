use crate::kernel::int::curve::chord::{Chord, SegmentChord};
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, Default)]
pub struct CubicSegment<I: IntNumber> {
    pub control_points: [IntPoint<I>; 4],
}

impl<I: IntNumber> Chord<I> for CubicSegment<I> {
    #[inline]
    fn chord(&self) -> SegmentChord<I> {
        SegmentChord {
            a: self.control_points[0],
            b: self.control_points[3],
        }
    }
}
