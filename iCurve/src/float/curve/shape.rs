use crate::float::curve::path::CurvePath;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

#[derive(Clone, PartialEq)]
pub struct CurveShape<P: FloatPointCompatible> {
    pub(crate) contours: Vec<CurvePath<P>>,
}

impl<P: FloatPointCompatible> CurveShape<P> {
    #[inline]
    pub fn contours(&self) -> &[CurvePath<P>] {
        &self.contours
    }

    /// Returns the total number of curve segments in the shape.
    pub fn segment_count(&self) -> usize {
        self.contours.iter().map(|path| path.segments.len()).sum()
    }

    pub(crate) fn bounds(&self) -> FloatRect<P::Scalar> {
        self.contours
            .iter()
            .map(CurvePath::bounds)
            .reduce(FloatRect::with_rects)
            .unwrap_or_else(FloatRect::zero)
    }
}
