use crate::int::CurveInt;
use crate::kernel::int::curve::chord::{Chord, SegmentChord};
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LineSegment<I: CurveInt> {
    pub(crate) control_points: [IntPoint<I>; 2],
}

impl<I: CurveInt> Chord<I> for LineSegment<I> {
    #[inline]
    fn chord(&self) -> SegmentChord<I> {
        SegmentChord {
            a: self.control_points[0],
            b: self.control_points[1],
        }
    }
}
