use crate::float::curve::arc::{
    EllipticArc, EllipticArcError, RationalArc, RationalArcError, is_finite_point,
};
use crate::float::curve::path::CurvePath;
use crate::float::curve::segment::CurveSegment;
use crate::float::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

pub struct CurveBuilder<P: FloatPointCompatible> {
    contours: Vec<CurvePath<P>>,
    current: Option<CurvePath<P>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveError {
    MissingMoveTo,
    EmptyPath,
    UnclosedContour,
    NoContours,
    NonFinitePoint,
    NonFiniteBounds,
    Arc(EllipticArcError),
    RationalArc(RationalArcError),
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

impl<P: FloatPointCompatible> Default for CurveBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: FloatPointCompatible> CurveBuilder<P> {
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
            current: None,
        }
    }

    pub fn move_to(mut self, point: P) -> Result<Self, CurveError> {
        validate_point(point)?;
        self.flush_current()?;
        self.current = Some(CurvePath {
            start: point,
            segments: Vec::new(),
        });
        Ok(self)
    }

    pub fn line_to(mut self, to: P) -> Result<Self, CurveError> {
        validate_point(to)?;
        self.push_segment(CurveSegment::Line { to })?;
        Ok(self)
    }

    pub fn quad_to(mut self, ctrl: P, to: P) -> Result<Self, CurveError> {
        validate_point(ctrl)?;
        validate_point(to)?;
        self.push_segment(CurveSegment::Quad { ctrl, to })?;
        Ok(self)
    }

    pub fn cubic_to(mut self, ctrl0: P, ctrl1: P, to: P) -> Result<Self, CurveError> {
        validate_point(ctrl0)?;
        validate_point(ctrl1)?;
        validate_point(to)?;
        self.push_segment(CurveSegment::Cubic { ctrl0, ctrl1, to })?;
        Ok(self)
    }

    pub fn arc_to(mut self, arc: EllipticArc<P>) -> Result<Self, CurveError> {
        for arc in arc.to_rational_arcs()? {
            self.push_segment(CurveSegment::Arc { arc })?;
        }
        Ok(self)
    }

    /// Appends an already materialized rational arc.
    ///
    /// This is primarily useful for feeding a previous boolean result back
    /// into another operation without reconstructing its supporting ellipse.
    pub fn rational_arc_to(mut self, arc: RationalArc<P>) -> Result<Self, CurveError> {
        arc.validate()?;
        self.push_segment(CurveSegment::Arc { arc })?;
        Ok(self)
    }

    /// Closes the current contour with a line when its endpoint differs from
    /// its start point, then commits it to the builder.
    pub fn close_contour(mut self) -> Result<Self, CurveError> {
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

    pub fn build(mut self) -> Result<CurveShape<P>, CurveError> {
        self.flush_current()?;
        if self.contours.is_empty() {
            return Err(CurveError::NoContours);
        }

        let shape = CurveShape {
            contours: self.contours,
        };
        let bounds = shape.bounds();
        let finite_bounds = bounds.min_x.to_f64().is_finite()
            && bounds.max_x.to_f64().is_finite()
            && bounds.min_y.to_f64().is_finite()
            && bounds.max_y.to_f64().is_finite()
            && bounds.width().to_f64().is_finite()
            && bounds.height().to_f64().is_finite();

        if finite_bounds {
            Ok(shape)
        } else {
            Err(CurveError::NonFiniteBounds)
        }
    }

    fn push_segment(&mut self, segment: CurveSegment<P>) -> Result<(), CurveError> {
        match self.current.as_mut() {
            Some(path) => {
                path.segments.push(segment);
                Ok(())
            }
            None => Err(CurveError::MissingMoveTo),
        }
    }

    fn flush_current(&mut self) -> Result<(), CurveError> {
        let Some(path) = self.current.take() else {
            return Ok(());
        };
        if path.segments.is_empty() {
            return Err(CurveError::EmptyPath);
        }
        if !path.is_closed() {
            return Err(CurveError::UnclosedContour);
        }
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
        assert!(shape.contours()[0].is_closed());
        assert!(shape.contours()[1].is_closed());
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
        let missing_move = CurveBuilder::<Point>::new().line_to([1.0, 0.0]);
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
        let point = CurveBuilder::<Point>::new().move_to([f64::NAN, 0.0]);
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
        let arc = CurveBuilder::new().move_to([1.0, 0.0])?.arc_to(invalid_arc);
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
        let arc = CurveBuilder::new().move_to([1.0, 0.0])?.arc_to(oversized_arc);
        assert!(matches!(
            arc,
            Err(CurveError::Arc(EllipticArcError::SweepTooLarge))
        ));
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

        let result = CurveBuilder::new()
            .move_to(rational.start_point())?
            .rational_arc_to(rational);

        assert!(matches!(
            result,
            Err(CurveError::RationalArc(RationalArcError::NonPositiveWeight))
        ));
        Ok(())
    }
}
