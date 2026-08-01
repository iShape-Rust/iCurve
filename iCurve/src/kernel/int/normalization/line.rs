use crate::int::CurveInt;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::segment::Segment;

impl<I: CurveInt> LineSegment<I> {
    #[inline]
    pub(crate) fn try_segment(self) -> Option<Segment<I>> {
        let [p0, p1] = self.control_points;

        // Same endpoint: zero-length edge.
        if p0 != p1 { Some(Segment::Line(self)) } else { None }
    }
}
