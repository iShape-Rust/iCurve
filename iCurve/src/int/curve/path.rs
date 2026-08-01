use crate::int::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurvePath<I: IntNumber> {
    pub start: IntPoint<I>,
    pub segments: Vec<CurveSegment<I>>,
}

impl<I: IntNumber> CurvePath<I> {
    /// Creates a path from a start point and connected segments.
    pub fn new(start: IntPoint<I>, segments: Vec<CurveSegment<I>>) -> Self {
        Self { start, segments }
    }

    /// Returns the final segment endpoint, or `None` for an empty path.
    pub fn end_point(&self) -> Option<IntPoint<I>> {
        self.segments.last().map(CurveSegment::end_point)
    }

    /// Returns whether this non-empty path ends at its start point.
    pub fn is_closed(&self) -> bool {
        self.end_point() == Some(self.start)
    }
}
