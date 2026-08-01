use crate::float::curve::arc::is_finite_point;
use crate::float::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;

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
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns whether this path contains no segments.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
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

    #[inline]
    pub fn end_point(&self) -> Option<P> {
        self.segments.last().map(CurveSegment::end_point)
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.end_point()
            .is_some_and(|end| end.x() == self.start.x() && end.y() == self.start.y())
    }

    pub(crate) fn bounds(&self) -> FloatRect<P::Scalar> {
        let mut bounds = None;
        add_point(&mut bounds, self.start);

        for segment in &self.segments {
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
