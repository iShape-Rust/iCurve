use crate::float::curve::arc::is_finite_point;
use crate::float::curve::builder::CurveError;
use crate::float::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::{FloatRect, FloatRectError};

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

    pub(crate) fn bounds(&self) -> Result<FloatRect<P::Scalar>, FloatRectError> {
        Self::bounds_for_parts(self.start, &self.segments)
    }

    fn bounds_for_parts(
        start: P,
        segments: &[CurveSegment<P>],
    ) -> Result<FloatRect<P::Scalar>, FloatRectError> {
        // Accumulate extrema without validating each input point. Only the
        // resulting rectangle is checked; callers must supply valid geometry.
        let mut bounds = FloatRect {
            min_x: start.x(),
            max_x: start.x(),
            min_y: start.y(),
            max_y: start.y(),
        };

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
                    let ellipse_bounds = arc.ellipse.bounds()?;
                    bounds.min_x = bounds.min_x.min(ellipse_bounds.min_x);
                    bounds.max_x = bounds.max_x.max(ellipse_bounds.max_x);
                    bounds.min_y = bounds.min_y.min(ellipse_bounds.min_y);
                    bounds.max_y = bounds.max_y.max(ellipse_bounds.max_y);
                    for point in arc.control_points {
                        add_point(&mut bounds, point);
                    }
                }
            }
        }

        FloatRect::new(bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y)
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
        Self::bounds_for_parts(start, segments)?;
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
fn add_point<P: FloatPointCompatible>(bounds: &mut FloatRect<P::Scalar>, point: P) {
    bounds.min_x = bounds.min_x.min(point.x());
    bounds.max_x = bounds.max_x.max(point.x());
    bounds.min_y = bounds.min_y.min(point.y());
    bounds.max_y = bounds.max_y.max(point.y());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float::curve::arc::{Ellipse, RationalArc};

    #[test]
    fn bounds_check_coordinate_limits_for_both_scalar_widths() {
        fn check<F: FloatNumber>() {
            let limit = F::MAX_COORDINATE;
            let start = [-limit, -limit];
            let segments = [CurveSegment::Line { to: [limit, limit] }];
            let bounds = CurvePath::bounds_for_parts(start, &segments).unwrap();
            assert!(bounds.min_x == -limit && bounds.max_x == limit);
            assert!(bounds.min_y == -limit && bounds.max_y == limit);

            let segments = [CurveSegment::Line {
                to: [limit * F::TWO, F::ZERO],
            }];
            assert_eq!(
                CurvePath::bounds_for_parts(start, &segments).err(),
                Some(FloatRectError::CoordinatesOutOfRange)
            );
        }
        check::<f32>();
        check::<f64>();
    }

    #[test]
    fn bounds_include_bezier_control_points() {
        let path = CurvePath::try_new(
            [0.0_f64, 0.0],
            alloc::vec![
                CurveSegment::Quad {
                    ctrl: [-5.0, 8.0],
                    to: [1.0, 1.0]
                },
                CurveSegment::Cubic {
                    ctrl0: [9.0, -7.0],
                    ctrl1: [2.0, 3.0],
                    to: [0.0, 0.0]
                },
            ],
        )
        .unwrap();
        let bounds = path.bounds().unwrap();
        assert_eq!(
            (bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y),
            (-5.0, 9.0, -7.0, 8.0)
        );
    }

    #[test]
    fn bounds_include_rotated_ellipse_and_authoritative_arc_controls() {
        let ellipse = Ellipse {
            center: [10.0_f64, 20.0],
            radius_x: 2.0,
            radius_y: 1.0,
            rotation: core::f64::consts::FRAC_PI_4,
        };
        let path = CurvePath::try_new(
            ellipse.center,
            alloc::vec![CurveSegment::Arc {
                arc: RationalArc {
                    ellipse,
                    control_points: [ellipse.center, [30.0, -10.0], ellipse.center],
                    weights: [1.0; 3],
                    start_angle: 0.0,
                    sweep_angle: 1.0,
                }
            }],
        )
        .unwrap();
        let bounds = path.bounds().unwrap();
        let extent = 2.5_f64.sqrt();
        assert!((bounds.min_x - (10.0 - extent)).abs() < 1.0e-12);
        assert!((bounds.max_y - (20.0 + extent)).abs() < 1.0e-12);
        assert_eq!(bounds.max_x, 30.0);
        assert_eq!(bounds.min_y, -10.0);
    }

    #[test]
    fn ellipse_bounds_error_reaches_path_constructor() {
        let ellipse = Ellipse {
            center: [f64::MAX_COORDINATE, 0.0],
            radius_x: f64::MAX_COORDINATE,
            radius_y: 1.0,
            rotation: 0.0,
        };
        let result = CurvePath::try_new(
            ellipse.center,
            alloc::vec![CurveSegment::Arc {
                arc: RationalArc {
                    ellipse,
                    control_points: [ellipse.center; 3],
                    weights: [1.0; 3],
                    start_angle: 0.0,
                    sweep_angle: 1.0,
                }
            }],
        );
        assert_eq!(
            result.err(),
            Some(CurveError::InvalidBounds(FloatRectError::CoordinatesOutOfRange))
        );
    }
}
