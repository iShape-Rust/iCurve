use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;

impl<I: IntNumber> LineSegment<I> {
    #[inline]
    pub(crate) fn try_segment(self) -> Option<Segment<I>> {
        let [p0, p1] = self.control_points;

        if p0 != p1 { Some(Segment::Line(self)) } else { None }
    }
}
