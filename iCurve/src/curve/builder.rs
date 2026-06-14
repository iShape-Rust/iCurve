use crate::curve::path::CurvePath;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

pub struct CurveBuilder<P: FloatPointCompatible> {
    paths: Vec<CurvePath<P>>,
    current: Option<CurvePath<P>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveError {
    MissingMoveTo,
    EmptyPath,
    UnclosedContour,
    NoContours,
}

impl<P: FloatPointCompatible> CurveBuilder<P> {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            current: None,
        }
    }

    pub fn move_to(mut self, point: P) -> Result<Self, CurveError> {
        self.flush_current()?;
        self.current = Some(CurvePath {
            start: point,
            segments: Vec::new(),
        });

        Ok(self)
    }

    pub fn line_to(mut self, to: P) -> Result<Self, CurveError> {
        self.push_segment(CurveSegment::Line { to })?;
        Ok(self)
    }

    pub fn quad_to(mut self, ctrl: P, to: P) -> Result<Self, CurveError> {
        self.push_segment(CurveSegment::Quad { ctrl, to })?;
        Ok(self)
    }

    pub fn cubic_to(mut self, ctrl0: P, ctrl1: P, to: P) -> Result<Self, CurveError> {
        self.push_segment(CurveSegment::Cubic { ctrl0, ctrl1, to })?;
        Ok(self)
    }

    pub fn close_contour(mut self) -> Result<Self, CurveError> {
        let Some(path) = self.current.as_mut() else {
            return Err(CurveError::MissingMoveTo);
        };

        if path.segments.is_empty() {
            return Err(CurveError::EmptyPath);
        }

        if let Some(end_point) = path.end_point() {
            if !same_point(end_point, path.start) {
                path.segments.push(CurveSegment::Line { to: path.start });
            }
        }

        self.flush_current()?;
        Ok(self)
    }

    pub fn build_shape(mut self) -> Result<CurveShape<P>, CurveError> {
        self.flush_current()?;

        if self.paths.is_empty() {
            Err(CurveError::NoContours)
        } else {
            Ok(CurveShape { contours: self.paths })
        }
    }

    pub fn build_path(mut self) -> Result<CurvePath<P>, CurveError> {
        if let Some(path) = self.current.take()
            && !path.segments.is_empty()
        {
            return Ok(path);
        }
        Err(CurveError::EmptyPath)
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
        self.paths.push(path);
        Ok(())
    }
}

impl<P: FloatPointCompatible> CurvePath<P> {
    fn is_closed(&self) -> bool {
        self.end_point()
            .is_some_and(|end_point| same_point(end_point, self.start))
    }

    fn end_point(&self) -> Option<P> {
        self.segments.last().map(CurveSegment::end_point)
    }
}

impl<P: FloatPointCompatible> CurveSegment<P> {
    fn end_point(&self) -> P {
        match self {
            Self::Line { to } | Self::Quad { to, .. } | Self::Cubic { to, .. } => *to,
        }
    }
}

fn same_point<P: FloatPointCompatible>(a: P, b: P) -> bool {
    a.x() == b.x() && a.y() == b.y()
}

impl<P: FloatPointCompatible> Default for CurveBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::segment::CurveSegment;

    #[test]
    fn build_shape_with_multiple_contours() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .quad_to([1.0, 1.0], [0.0, 1.0])?
            .close_contour()?
            .move_to([2.0, 2.0])?
            .cubic_to([3.0, 2.0], [3.0, 3.0], [2.0, 3.0])?
            .close_contour()?
            .build_shape()?;

        assert_eq!(shape.contours.len(), 2);
        assert_eq!(shape.contours[0].start, [0.0, 0.0]);
        assert_eq!(shape.contours[0].segments.len(), 3);
        assert_eq!(shape.contours[1].start, [2.0, 2.0]);
        assert_eq!(shape.contours[1].segments.len(), 2);

        Ok(())
    }

    #[test]
    fn build_closed_polygon_preserves_explicit_closing_endpoint() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([-210.0_f32, -130.0])?
            .line_to([70.0, -130.0])?
            .line_to([70.0, 130.0])?
            .line_to([-216.049_59, 129.983_02])?
            .line_to([-210.0, -130.0])?
            .build_shape()?;

        assert_eq!(shape.contours.len(), 1);
        let contour = &shape.contours[0];
        assert_eq!(contour.start, [-210.0, -130.0]);
        match contour.segments.last() {
            Some(CurveSegment::Line { to }) => assert_eq!(*to, contour.start),
            _ => panic!("expected closing line segment"),
        }

        Ok(())
    }

    #[test]
    fn segment_without_move_to_is_error() {
        let result = CurveBuilder::<[f64; 2]>::new().line_to([1.0, 0.0]);

        assert!(matches!(result, Err(CurveError::MissingMoveTo)));
    }

    #[test]
    fn close_with_line_closes_open_contour() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .close_contour()?
            .build_shape()?;

        assert_eq!(shape.contours.len(), 1);
        let contour = &shape.contours[0];
        assert_eq!(contour.segments.len(), 2);
        match contour.segments.last() {
            Some(CurveSegment::Line { to }) => assert_eq!(*to, contour.start),
            _ => panic!("expected closing line segment"),
        }

        Ok(())
    }

    #[test]
    fn build_requires_closed_contour() -> Result<(), CurveError> {
        let result = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .build_shape();

        assert!(matches!(result, Err(CurveError::UnclosedContour)));
        Ok(())
    }

    #[test]
    fn empty_contour_is_error() -> Result<(), CurveError> {
        let result = CurveBuilder::<[f64; 2]>::new().move_to([0.0, 0.0])?.build_shape();

        assert!(matches!(result, Err(CurveError::EmptyPath)));
        Ok(())
    }
}
