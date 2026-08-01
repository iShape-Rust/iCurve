use crate::float::curve::builder::CurveError;
use crate::float::curve::path::CurvePath;
use crate::float::curve::path::finite_rect;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

/// A validated, non-empty collection of closed curve contours.
///
/// Every contour satisfies the [`CurvePath`] invariants and the combined
/// bounds are finite. Contour orientation, nesting, and intersections are not
/// constrained. Empty geometry is represented by an empty collection of
/// shapes rather than an empty `CurveShape`.
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
    /// Creates a validated shape from closed contours.
    pub fn try_new(contours: Vec<CurvePath<P>>) -> Result<Self, CurveError> {
        Self::validate_contours(&contours)?;
        Ok(Self { contours })
    }

    pub(crate) fn from_validated_contours(contours: Vec<CurvePath<P>>) -> Self {
        debug_assert!(Self::validate_contours(&contours).is_ok());
        Self { contours }
    }

    /// Creates a shape containing one validated closed path.
    #[inline]
    pub fn from_path(path: CurvePath<P>) -> Self {
        Self {
            contours: alloc::vec![path],
        }
    }

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
    #[allow(
        clippy::len_without_is_empty,
        reason = "a validated curve shape is never empty"
    )]
    pub fn len(&self) -> usize {
        self.contours.len()
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

    pub(crate) fn validate_contours(contours: &[CurvePath<P>]) -> Result<(), CurveError> {
        if contours.is_empty() {
            return Err(CurveError::NoContours);
        }
        for contour in contours {
            contour.validate()?;
        }
        let bounds = contours
            .iter()
            .map(CurvePath::bounds)
            .reduce(FloatRect::with_rects)
            .unwrap_or_else(FloatRect::zero);
        if !finite_rect(&bounds) {
            return Err(CurveError::NonFiniteBounds);
        }
        Ok(())
    }
}

impl<P: FloatPointCompatible> From<CurvePath<P>> for CurveShape<P> {
    #[inline]
    fn from(path: CurvePath<P>) -> Self {
        Self::from_path(path)
    }
}

impl<P: FloatPointCompatible> TryFrom<Vec<CurvePath<P>>> for CurveShape<P> {
    type Error = CurveError;

    #[inline]
    fn try_from(contours: Vec<CurvePath<P>>) -> Result<Self, Self::Error> {
        Self::try_new(contours)
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
