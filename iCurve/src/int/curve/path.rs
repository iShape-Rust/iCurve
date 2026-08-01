use crate::int::CurveInt;
use crate::int::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_shape::int::IntPoint;

/// Integer curve contour represented by a start point and ordered segments.
///
/// Construction does not validate the contour. Boolean input requires at
/// least one segment, exact closure, and valid connected rational arcs; these
/// conditions are checked by [`IntCurveOverlay`](crate::int::IntCurveOverlay).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurvePath<I: CurveInt> {
    /// First point of the contour and implicit start point of its first segment.
    pub start: IntPoint<I>,
    /// Ordered segments whose endpoints advance around the contour.
    pub segments: Vec<CurveSegment<I>>,
}

impl<I: CurveInt> CurvePath<I> {
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
