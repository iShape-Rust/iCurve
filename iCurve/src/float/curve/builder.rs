use crate::float::curve::arc::{
    EllipticArc, EllipticArcError, RationalArc, RationalArcError, is_finite_point,
};
use crate::float::curve::path::{CurvePath, same_point};
use crate::float::curve::segment::CurveSegment;
use crate::float::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

/// Mutable builder for closed float curve paths.
///
/// Path commands return this builder by mutable reference, so they can be
/// chained or called repeatedly from loops. A successful [`build`](Self::build)
/// resets the builder for reuse.
pub struct CurveBuilder<P: FloatPointCompatible> {
    contours: Vec<CurvePath<P>>,
    current: Option<PathDraft<P>>,
}

struct PathDraft<P: FloatPointCompatible> {
    start: P,
    segments: Vec<CurveSegment<P>>,
}

impl<P: FloatPointCompatible> PathDraft<P> {
    #[inline]
    fn from_path(path: CurvePath<P>) -> Self {
        let (start, segments) = path.into_parts();
        Self { start, segments }
    }

    #[inline]
    fn current_point(&self) -> P {
        self.segments
            .last()
            .map(CurveSegment::end_point)
            .unwrap_or(self.start)
    }

    #[inline]
    fn is_closed(&self) -> bool {
        self.segments
            .last()
            .is_some_and(|segment| same_point(segment.end_point(), self.start))
    }

    #[inline]
    fn validate(&self) -> Result<(), CurveError> {
        CurvePath::validate_parts(self.start, &self.segments)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CurveError {
    MissingMoveTo,
    EmptyPath,
    UnclosedContour,
    NoContours,
    NonFinitePoint,
    NonFiniteBounds,
    Arc(EllipticArcError),
    RationalArc(RationalArcError),
    DisconnectedArc,
}

impl From<EllipticArcError> for CurveError {
    fn from(error: EllipticArcError) -> Self {
        Self::Arc(error)
    }
}

impl From<RationalArcError> for CurveError {
    fn from(error: RationalArcError) -> Self {
        Self::RationalArc(error)
    }
}

impl core::fmt::Display for CurveError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingMoveTo => formatter.write_str("a segment requires move_to first"),
            Self::EmptyPath => formatter.write_str("a contour must contain at least one segment"),
            Self::UnclosedContour => formatter.write_str("all curve contours must be closed"),
            Self::NoContours => formatter.write_str("a curve shape must contain at least one contour"),
            Self::NonFinitePoint => formatter.write_str("curve points must be finite"),
            Self::NonFiniteBounds => formatter.write_str("curve bounds must be finite"),
            Self::Arc(_) => formatter.write_str("invalid elliptic arc"),
            Self::RationalArc(_) => formatter.write_str("invalid rational arc"),
            Self::DisconnectedArc => formatter.write_str("an arc must start at the current path point"),
        }
    }
}

impl core::error::Error for CurveError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Arc(error) => Some(error),
            Self::RationalArc(error) => Some(error),
            _ => None,
        }
    }
}

impl<P: FloatPointCompatible> Default for CurveBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: FloatPointCompatible> CurveBuilder<P> {
    /// Creates an empty curve builder.
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
            current: None,
        }
    }

    /// Starts a new contour after committing the preceding closed contour.
    pub fn move_to(&mut self, point: P) -> Result<&mut Self, CurveError> {
        validate_point(point)?;
        self.flush_current()?;
        self.current = Some(PathDraft {
            start: point,
            segments: Vec::new(),
        });
        Ok(self)
    }

    /// Appends a line segment to the current contour.
    pub fn line_to(&mut self, to: P) -> Result<&mut Self, CurveError> {
        validate_point(to)?;
        self.push_segment(CurveSegment::Line { to })?;
        Ok(self)
    }

    /// Appends a quadratic Bézier segment to the current contour.
    pub fn quad_to(&mut self, ctrl: P, to: P) -> Result<&mut Self, CurveError> {
        validate_point(ctrl)?;
        validate_point(to)?;
        self.push_segment(CurveSegment::Quad { ctrl, to })?;
        Ok(self)
    }

    /// Appends a cubic Bézier segment to the current contour.
    pub fn cubic_to(&mut self, ctrl0: P, ctrl1: P, to: P) -> Result<&mut Self, CurveError> {
        validate_point(ctrl0)?;
        validate_point(ctrl1)?;
        validate_point(to)?;
        self.push_segment(CurveSegment::Cubic { ctrl0, ctrl1, to })?;
        Ok(self)
    }

    /// Appends an elliptic arc as connected rational quadratic segments.
    pub fn arc_to(&mut self, arc: EllipticArc<P>) -> Result<&mut Self, CurveError> {
        for arc in arc.to_rational_arcs()? {
            self.push_segment(CurveSegment::Arc { arc })?;
        }
        Ok(self)
    }

    /// Appends an already materialized rational arc.
    ///
    /// This is primarily useful for feeding a previous boolean result back
    /// into another operation without reconstructing its supporting ellipse.
    pub fn rational_arc_to(&mut self, arc: RationalArc<P>) -> Result<&mut Self, CurveError> {
        arc.validate()?;
        self.push_segment(CurveSegment::Arc { arc })?;
        Ok(self)
    }

    /// Closes the current contour with a line when its endpoint differs from
    /// its start point, then commits it to the builder.
    pub fn close_contour(&mut self) -> Result<&mut Self, CurveError> {
        let Some(path) = self.current.as_mut() else {
            return Err(CurveError::MissingMoveTo);
        };
        if path.segments.is_empty() {
            return Err(CurveError::EmptyPath);
        }

        if !path.is_closed() {
            path.segments.push(CurveSegment::Line { to: path.start });
        }

        self.flush_current()?;
        Ok(self)
    }

    /// Builds a shape and resets this builder after success.
    pub fn build(&mut self) -> Result<CurveShape<P>, CurveError> {
        let had_current = self.current.is_some();
        self.flush_current()?;
        if self.contours.is_empty() {
            return Err(CurveError::NoContours);
        }

        if let Err(error) = CurveShape::validate_contours(&self.contours) {
            if had_current {
                let path = self.contours.pop().expect("current path was flushed above");
                self.current = Some(PathDraft::from_path(path));
            }
            Err(error)
        } else {
            Ok(CurveShape::from_validated_contours(core::mem::take(
                &mut self.contours,
            )))
        }
    }

    fn push_segment(&mut self, segment: CurveSegment<P>) -> Result<(), CurveError> {
        match self.current.as_mut() {
            Some(path) => {
                if let CurveSegment::Arc { arc } = &segment {
                    let current = path.current_point();
                    if !same_point(current, arc.start_point()) {
                        return Err(CurveError::DisconnectedArc);
                    }
                }
                path.segments.push(segment);
                Ok(())
            }
            None => Err(CurveError::MissingMoveTo),
        }
    }

    fn flush_current(&mut self) -> Result<(), CurveError> {
        let Some(path) = self.current.as_ref() else {
            return Ok(());
        };
        path.validate()?;
        let path = self.current.take().expect("current path was validated above");
        let path = CurvePath::from_validated_parts(path.start, path.segments);
        self.contours.push(path);
        Ok(())
    }
}

#[inline]
fn validate_point<P: FloatPointCompatible>(point: P) -> Result<(), CurveError> {
    if is_finite_point(point) {
        Ok(())
    } else {
        Err(CurveError::NonFinitePoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float::curve::arc::{Ellipse, EllipticArcError, RationalArcError};

    type Point = [f64; 2];

    #[test]
    fn builds_multiple_closed_float_contours_without_quantization() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.25, 0.5])?
            .line_to([2.5, 0.5])?
            .quad_to([3.75, 2.25], [0.25, 2.5])?
            .close_contour()?
            .move_to([10.125, 10.25])?
            .cubic_to([12.5, 10.25], [12.5, 12.75], [10.125, 10.25])?
            .build()?;

        assert_eq!(shape.contours().len(), 2);
        assert_eq!(shape.contours()[0].start(), [0.25, 0.5]);
        assert_eq!(shape.contours()[0].segments().len(), 3);
        Ok(())
    }

    #[test]
    fn close_contour_adds_only_the_required_line() -> Result<(), CurveError> {
        let open = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .close_contour()?
            .build()?;
        let already_closed = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([0.0, 0.0])?
            .close_contour()?
            .build()?;

        assert_eq!(open.contours()[0].segments().len(), 2);
        assert_eq!(already_closed.contours()[0].segments().len(), 1);
        Ok(())
    }

    #[test]
    fn rejects_structurally_invalid_paths() -> Result<(), CurveError> {
        let mut builder = CurveBuilder::<Point>::new();
        let missing_move = builder.line_to([1.0, 0.0]);
        assert!(matches!(missing_move, Err(CurveError::MissingMoveTo)));

        let empty = CurveBuilder::<Point>::new().move_to([0.0, 0.0])?.build();
        assert!(matches!(empty, Err(CurveError::EmptyPath)));

        let unclosed = CurveBuilder::<Point>::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .build();
        assert!(matches!(unclosed, Err(CurveError::UnclosedContour)));
        Ok(())
    }

    #[test]
    fn rejects_non_finite_points_and_arcs() -> Result<(), CurveError> {
        let mut builder = CurveBuilder::<Point>::new();
        let point = builder.move_to([f64::NAN, 0.0]);
        assert!(matches!(point, Err(CurveError::NonFinitePoint)));

        let bounds = CurveBuilder::<Point>::new()
            .move_to([-f64::MAX, 0.0])?
            .line_to([f64::MAX, 0.0])?
            .line_to([-f64::MAX, 0.0])?
            .build();
        assert!(matches!(bounds, Err(CurveError::NonFiniteBounds)));

        let invalid_arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0, 0.0],
                radius_x: 0.0,
                radius_y: 1.0,
                rotation: 0.0,
            },
            start_angle: 0.0,
            sweep_angle: 1.0,
        };
        let mut builder = CurveBuilder::new();
        builder.move_to([1.0, 0.0])?;
        let arc = builder.arc_to(invalid_arc);
        assert!(matches!(
            arc,
            Err(CurveError::Arc(EllipticArcError::NonPositiveRadius))
        ));

        let oversized_arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0, 0.0],
                radius_x: 1.0,
                radius_y: 1.0,
                rotation: 0.0,
            },
            start_angle: 0.0,
            sweep_angle: core::f64::consts::TAU + 0.1,
        };
        let mut builder = CurveBuilder::new();
        builder.move_to([1.0, 0.0])?;
        let arc = builder.arc_to(oversized_arc);
        assert!(matches!(
            arc,
            Err(CurveError::Arc(EllipticArcError::SweepTooLarge))
        ));
        Ok(())
    }

    #[test]
    fn nested_curve_errors_preserve_the_full_source_chain() {
        let error = CurveError::RationalArc(RationalArcError::Elliptic(EllipticArcError::NonPositiveRadius));

        assert_eq!(alloc::format!("{error}"), "invalid rational arc");
        let rational_source = core::error::Error::source(&error).unwrap();
        assert!(rational_source.is::<RationalArcError>());
        assert_eq!(
            alloc::format!("{rational_source}"),
            "invalid supporting elliptic arc"
        );

        let elliptic_source = rational_source.source().unwrap();
        assert!(elliptic_source.is::<EllipticArcError>());
        assert_eq!(
            alloc::format!("{elliptic_source}"),
            "ellipse radii must be positive"
        );
        assert!(elliptic_source.source().is_none());

        let direct = CurveError::Arc(EllipticArcError::ZeroSweep);
        assert!(core::error::Error::source(&direct).is_some_and(|source| source.is::<EllipticArcError>()));
    }

    #[test]
    fn rejects_disconnected_arcs_in_builder_and_path_constructor() -> Result<(), CurveError> {
        let arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0, 0.0],
                radius_x: 5.0,
                radius_y: 5.0,
                rotation: 0.0,
            },
            start_angle: 0.0,
            sweep_angle: core::f64::consts::FRAC_PI_2,
        };

        let mut builder = CurveBuilder::new();
        let error = builder.move_to([0.0, 0.0])?.arc_to(arc);
        assert!(matches!(error, Err(CurveError::DisconnectedArc)));

        let rational = arc.to_rational_arcs()?.remove(0);
        let error = CurvePath::try_new([0.0, 0.0], alloc::vec![CurveSegment::Arc { arc: rational }]);
        assert!(matches!(error, Err(CurveError::DisconnectedArc)));
        Ok(())
    }

    #[test]
    fn stores_valid_arc_as_connected_rational_pieces() -> Result<(), CurveError> {
        let arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0, 0.0],
                radius_x: 2.0,
                radius_y: 1.0,
                rotation: 0.0,
            },
            start_angle: 0.0,
            sweep_angle: core::f64::consts::TAU,
        };
        let shape = CurveBuilder::new()
            .move_to(arc.start_point())?
            .arc_to(arc)?
            .close_contour()?
            .build()?;

        assert_eq!(shape.contours().len(), 1);
        let segments = shape.contours()[0].segments();
        assert_eq!(segments.len(), 4);

        let mut current = shape.contours()[0].start();
        for segment in segments {
            let CurveSegment::Arc { arc } = segment else {
                panic!("expected rational arc");
            };
            assert_eq!(arc.start_point(), current);
            assert!(arc.supporting_arc().ellipse == arc.ellipse);
            current = arc.end_point();
        }
        assert_eq!(current, shape.contours()[0].start());
        Ok(())
    }

    #[test]
    fn rejects_non_positive_rational_arc_weights() -> Result<(), CurveError> {
        let arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0, 0.0],
                radius_x: 2.0,
                radius_y: 1.0,
                rotation: 0.0,
            },
            start_angle: 0.0,
            sweep_angle: core::f64::consts::FRAC_PI_2,
        };
        let mut rational = arc.to_rational_arcs()?.remove(0);
        rational.weights[1] = 0.0;

        let mut builder = CurveBuilder::new();
        builder.move_to(rational.start_point())?;
        let result = builder.rational_arc_to(rational);

        assert!(matches!(
            result,
            Err(CurveError::RationalArc(RationalArcError::NonPositiveWeight))
        ));
        Ok(())
    }

    #[test]
    fn supports_dynamic_loops_and_reuse() -> Result<(), CurveError> {
        let mut builder = CurveBuilder::new();
        let points = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];

        builder.move_to(points[0])?;
        for point in &points[1..] {
            builder.line_to(*point)?;
        }
        let first = builder.close_contour()?.build()?;

        assert_eq!(first.contours().len(), 1);
        assert!(matches!(builder.build(), Err(CurveError::NoContours)));

        let second = builder
            .move_to([10.0, 10.0])?
            .line_to([11.0, 10.0])?
            .close_contour()?
            .build()?;

        assert_eq!(second.contours().len(), 1);
        assert_eq!(second.contours()[0].start(), [10.0, 10.0]);
        Ok(())
    }

    #[test]
    fn errors_preserve_mutable_builder_state() -> Result<(), CurveError> {
        let mut builder = CurveBuilder::new();
        builder.move_to([0.0, 0.0])?.line_to([1.0, 0.0])?;

        assert!(matches!(
            builder.move_to([10.0, 10.0]),
            Err(CurveError::UnclosedContour)
        ));
        assert!(matches!(builder.build(), Err(CurveError::UnclosedContour)));
        assert!(matches!(
            builder.line_to([f64::NAN, 0.0]),
            Err(CurveError::NonFinitePoint)
        ));

        let shape = builder.close_contour()?.build()?;
        assert_eq!(shape.contours()[0].start(), [0.0, 0.0]);
        assert_eq!(shape.contours()[0].segments().len(), 2);
        Ok(())
    }
}
