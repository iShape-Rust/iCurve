use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

use crate::curve::arc::EllipticArc;
use crate::curve::contour::CurveContour;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;

pub struct CurveShapeBuilder<P: FloatPointCompatible> {
    contours: Vec<CurveContour<P>>,
    current: Option<CurveContour<P>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveError {
    MissingMoveTo,
    EmptyContour,
}

impl<P: FloatPointCompatible> CurveShapeBuilder<P> {
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
            current: None,
        }
    }

    pub fn move_to(mut self, point: P) -> Result<Self, CurveError> {
        self.flush_current()?;
        self.current = Some(CurveContour {
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

    pub fn arc_to(mut self, arc: EllipticArc<P>) -> Result<Self, CurveError> {
        self.push_segment(CurveSegment::Arc { arc })?;
        Ok(self)
    }

    pub fn close(mut self) -> Result<Self, CurveError> {
        if self.current.is_none() {
            return Err(CurveError::MissingMoveTo);
        }

        self.flush_current()?;
        Ok(self)
    }

    pub fn build(mut self) -> Result<CurveShape<P>, CurveError> {
        self.flush_current()?;

        Ok(CurveShape {
            contours: self.contours,
        })
    }

    fn push_segment(&mut self, segment: CurveSegment<P>) -> Result<(), CurveError> {
        match self.current.as_mut() {
            Some(contour) => {
                contour.segments.push(segment);
                Ok(())
            }
            None => Err(CurveError::MissingMoveTo),
        }
    }

    fn flush_current(&mut self) -> Result<(), CurveError> {
        let Some(contour) = self.current.take() else {
            return Ok(());
        };

        if contour.segments.is_empty() {
            return Err(CurveError::EmptyContour);
        }

        self.contours.push(contour);
        Ok(())
    }
}

impl<P: FloatPointCompatible> Default for CurveShapeBuilder<P> {
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
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .quad_to([1.0, 1.0], [0.0, 1.0])?
            .close()?
            .move_to([2.0, 2.0])?
            .cubic_to([3.0, 2.0], [3.0, 3.0], [2.0, 3.0])?
            .arc_to(EllipticArc {
                center: [2.0, 2.0],
                radii: [1.0, 1.0],
                rotation: 0.0,
                start_angle: 0.0,
                sweep_angle: 1.0,
            })?
            .build()?;

        assert_eq!(shape.contours.len(), 2);
        assert_eq!(shape.contours[0].start, [0.0, 0.0]);
        assert_eq!(shape.contours[0].segments.len(), 2);
        assert_eq!(shape.contours[1].start, [2.0, 2.0]);
        assert_eq!(shape.contours[1].segments.len(), 2);

        Ok(())
    }

    #[test]
    fn build_closed_polygon_preserves_explicit_closing_endpoint() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([-210.0_f32, -130.0])?
            .line_to([70.0, -130.0])?
            .line_to([70.0, 130.0])?
            .line_to([-216.049_59, 129.983_02])?
            .line_to([-210.0, -130.0])?
            .build()?;

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
        let result = CurveShapeBuilder::<[f64; 2]>::new().line_to([1.0, 0.0]);

        assert!(matches!(result, Err(CurveError::MissingMoveTo)));
    }

    #[test]
    fn empty_contour_is_error() -> Result<(), CurveError> {
        let result = CurveShapeBuilder::<[f64; 2]>::new().move_to([0.0, 0.0])?.build();

        assert!(matches!(result, Err(CurveError::EmptyContour)));
        Ok(())
    }
}
