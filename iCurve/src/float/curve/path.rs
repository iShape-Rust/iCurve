use crate::float::curve::arc::is_finite_point;
use crate::float::curve::builder::CurveError;
use crate::float::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

/// A validated, non-empty closed curve contour.
///
/// All coordinates and bounds are finite, rational arcs are connected to the
/// preceding endpoint, and the final endpoint exactly equals [`start`](Self::start).
/// Degenerate segments and self-intersections are allowed.
#[derive(Clone, PartialEq)]
pub struct CurvePath<P: FloatPointCompatible> {
    pub(crate) start: P,
    pub(crate) segments: Vec<CurveSegment<P>>,
}

impl<P> core::fmt::Debug for CurvePath<P>
where
    P: FloatPointCompatible + core::fmt::Debug,
    P::Scalar: core::fmt::Debug,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CurvePath")
            .field("start", &self.start)
            .field("segments", &self.segments)
            .finish()
    }
}

impl<P: FloatPointCompatible> CurvePath<P> {
    /// Creates a validated closed path from a start point and connected segments.
    pub fn try_new(start: P, segments: Vec<CurveSegment<P>>) -> Result<Self, CurveError> {
        Self::validate_parts(start, &segments)?;
        Ok(Self { start, segments })
    }

    pub(crate) fn from_validated_parts(start: P, segments: Vec<CurveSegment<P>>) -> Self {
        debug_assert!(Self::validate_parts(start, &segments).is_ok());
        Self { start, segments }
    }

    /// Returns the first point of this path.
    #[inline]
    pub fn start(&self) -> P {
        self.start
    }

    /// Returns the segments in this path.
    #[inline]
    pub fn segments(&self) -> &[CurveSegment<P>] {
        &self.segments
    }

    /// Returns an iterator over the segments in this path.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, CurveSegment<P>> {
        self.segments.iter()
    }

    /// Returns the number of segments in this path.
    #[inline]
    #[allow(
        clippy::len_without_is_empty,
        reason = "a validated curve path is never empty"
    )]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Consumes this path and returns its start point and segments.
    #[inline]
    pub fn into_parts(self) -> (P, Vec<CurveSegment<P>>) {
        (self.start, self.segments)
    }

    /// Consumes this path and returns its segments without cloning.
    #[inline]
    pub fn into_segments(self) -> Vec<CurveSegment<P>> {
        self.segments
    }

    pub(crate) fn bounds(&self) -> FloatRect<P::Scalar> {
        Self::bounds_for_parts(self.start, &self.segments)
    }

    fn bounds_for_parts(start: P, segments: &[CurveSegment<P>]) -> FloatRect<P::Scalar> {
        let mut bounds = None;
        add_point(&mut bounds, start);

        for segment in segments {
            match segment {
                CurveSegment::Line { to } => add_point(&mut bounds, *to),
                CurveSegment::Quad { ctrl, to } => {
                    add_point(&mut bounds, *ctrl);
                    add_point(&mut bounds, *to);
                }
                CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                    add_point(&mut bounds, *ctrl0);
                    add_point(&mut bounds, *ctrl1);
                    add_point(&mut bounds, *to);
                }
                CurveSegment::Arc { arc } => {
                    let ellipse_bounds = arc.ellipse.bounds();
                    bounds = Some(match bounds {
                        Some(bounds) => FloatRect::with_rects(bounds, ellipse_bounds),
                        None => ellipse_bounds,
                    });
                    for point in arc.control_points {
                        add_point(&mut bounds, point);
                    }
                }
            }
        }

        bounds.unwrap_or_else(FloatRect::zero)
    }

    pub(crate) fn validate(&self) -> Result<(), CurveError> {
        Self::validate_parts(self.start, &self.segments)
    }

    pub(crate) fn validate_parts(start: P, segments: &[CurveSegment<P>]) -> Result<(), CurveError> {
        validate_point(start)?;
        if segments.is_empty() {
            return Err(CurveError::EmptyPath);
        }

        let mut current = start;
        for segment in segments {
            match segment {
                CurveSegment::Line { to } => validate_point(*to)?,
                CurveSegment::Quad { ctrl, to } => {
                    validate_point(*ctrl)?;
                    validate_point(*to)?;
                }
                CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                    validate_point(*ctrl0)?;
                    validate_point(*ctrl1)?;
                    validate_point(*to)?;
                }
                CurveSegment::Arc { arc } => {
                    arc.validate()?;
                    if !same_point(current, arc.start_point()) {
                        return Err(CurveError::DisconnectedArc);
                    }
                }
            }
            current = segment.end_point();
        }

        if !same_point(current, start) {
            return Err(CurveError::UnclosedContour);
        }
        if !finite_rect(&Self::bounds_for_parts(start, segments)) {
            return Err(CurveError::NonFiniteBounds);
        }
        Ok(())
    }
}

impl<P: FloatPointCompatible> TryFrom<(P, Vec<CurveSegment<P>>)> for CurvePath<P> {
    type Error = CurveError;

    #[inline]
    fn try_from((start, segments): (P, Vec<CurveSegment<P>>)) -> Result<Self, Self::Error> {
        Self::try_new(start, segments)
    }
}

impl<P: FloatPointCompatible> AsRef<[CurveSegment<P>]> for CurvePath<P> {
    #[inline]
    fn as_ref(&self) -> &[CurveSegment<P>] {
        &self.segments
    }
}

impl<P: FloatPointCompatible> IntoIterator for CurvePath<P> {
    type Item = CurveSegment<P>;
    type IntoIter = alloc::vec::IntoIter<CurveSegment<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}

impl<'a, P: FloatPointCompatible> IntoIterator for &'a CurvePath<P> {
    type Item = &'a CurveSegment<P>;
    type IntoIter = core::slice::Iter<'a, CurveSegment<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.segments.iter()
    }
}

#[inline]
fn add_point<P: FloatPointCompatible>(bounds: &mut Option<FloatRect<P::Scalar>>, point: P) {
    debug_assert!(is_finite_point(point));
    FloatRect::optional_add_point(bounds, &point);
}

#[inline]
fn validate_point<P: FloatPointCompatible>(point: P) -> Result<(), CurveError> {
    if is_finite_point(point) {
        Ok(())
    } else {
        Err(CurveError::NonFinitePoint)
    }
}

#[inline]
pub(crate) fn same_point<P: FloatPointCompatible>(a: P, b: P) -> bool {
    a.x() == b.x() && a.y() == b.y()
}

#[inline]
pub(crate) fn finite_rect<F: FloatNumber>(rect: &FloatRect<F>) -> bool {
    rect.min_x.to_f64().is_finite()
        && rect.max_x.to_f64().is_finite()
        && rect.min_y.to_f64().is_finite()
        && rect.max_y.to_f64().is_finite()
        && rect.width().to_f64().is_finite()
        && rect.height().to_f64().is_finite()
}
