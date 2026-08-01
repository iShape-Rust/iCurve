use crate::float::curve::path::CurvePath;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

#[derive(Clone, PartialEq)]
pub struct CurveShape<P: FloatPointCompatible> {
    pub(crate) contours: Vec<CurvePath<P>>,
}

impl<P> core::fmt::Debug for CurveShape<P>
where
    P: FloatPointCompatible + core::fmt::Debug,
    P::Scalar: core::fmt::Debug,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CurveShape")
            .field("contours", &self.contours)
            .finish()
    }
}

impl<P: FloatPointCompatible> CurveShape<P> {
    /// Returns the contours in this shape.
    #[inline]
    pub fn contours(&self) -> &[CurvePath<P>] {
        &self.contours
    }

    /// Returns an iterator over the contours in this shape.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, CurvePath<P>> {
        self.contours.iter()
    }

    /// Returns the number of contours in this shape.
    #[inline]
    pub fn len(&self) -> usize {
        self.contours.len()
    }

    /// Returns whether this shape contains no contours.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.contours.is_empty()
    }

    /// Consumes this shape and returns its contours without cloning.
    #[inline]
    pub fn into_contours(self) -> Vec<CurvePath<P>> {
        self.contours
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

impl<P: FloatPointCompatible> AsRef<[CurvePath<P>]> for CurveShape<P> {
    #[inline]
    fn as_ref(&self) -> &[CurvePath<P>] {
        &self.contours
    }
}

impl<P: FloatPointCompatible> IntoIterator for CurveShape<P> {
    type Item = CurvePath<P>;
    type IntoIter = alloc::vec::IntoIter<CurvePath<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.contours.into_iter()
    }
}

impl<'a, P: FloatPointCompatible> IntoIterator for &'a CurveShape<P> {
    type Item = &'a CurvePath<P>;
    type IntoIter = core::slice::Iter<'a, CurvePath<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.contours.iter()
    }
}
