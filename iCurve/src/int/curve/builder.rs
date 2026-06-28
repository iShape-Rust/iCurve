use crate::int::curve::path::CurvePath;
use crate::int::curve::segment::CurveSegment;
use crate::int::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub struct CurveBuilder<P: FloatPointCompatible, I: IntNumber> {
    adapter: FloatPointAdapter<P, I>,
    paths: Vec<CurvePath<I>>,
    current: Option<CurvePath<I>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveBuilderError {
    Curve(CurveError),
    Adapter(FloatPointAdapterRangeError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveError {
    MissingMoveTo,
    EmptyPath,
    UnclosedContour,
    NoContours,
}

impl From<CurveError> for CurveBuilderError {
    fn from(error: CurveError) -> Self {
        Self::Curve(error)
    }
}

impl From<FloatPointAdapterRangeError> for CurveBuilderError {
    fn from(error: FloatPointAdapterRangeError) -> Self {
        Self::Adapter(error)
    }
}

impl<P: FloatPointCompatible, I: IntNumber> CurveBuilder<P, I> {
    pub fn new(adapter: FloatPointAdapter<P, I>) -> Self {
        Self::with_adapter(adapter)
    }

    pub fn with_adapter(adapter: FloatPointAdapter<P, I>) -> Self {
        Self {
            adapter,
            paths: Vec::new(),
            current: None,
        }
    }

    pub fn adapter(&self) -> &FloatPointAdapter<P, I> {
        &self.adapter
    }

    pub fn move_to(mut self, point: P) -> Result<Self, CurveBuilderError> {
        self.flush_current()?;
        let start = self.adapter.try_float_to_int(&point)?;
        self.current = Some(CurvePath {
            start,
            segments: Vec::new(),
        });

        Ok(self)
    }

    pub fn line_to(mut self, to: P) -> Result<Self, CurveBuilderError> {
        let to = self.adapter.try_float_to_int(&to)?;
        self.push_segment(CurveSegment::Line { to })?;
        Ok(self)
    }

    pub fn quad_to(mut self, ctrl: P, to: P) -> Result<Self, CurveBuilderError> {
        let ctrl = self.adapter.try_float_to_int(&ctrl)?;
        let to = self.adapter.try_float_to_int(&to)?;
        self.push_segment(CurveSegment::Quad { ctrl, to })?;
        Ok(self)
    }

    pub fn cubic_to(mut self, ctrl0: P, ctrl1: P, to: P) -> Result<Self, CurveBuilderError> {
        let ctrl0 = self.adapter.try_float_to_int(&ctrl0)?;
        let ctrl1 = self.adapter.try_float_to_int(&ctrl1)?;
        let to = self.adapter.try_float_to_int(&to)?;
        self.push_segment(CurveSegment::Cubic { ctrl0, ctrl1, to })?;
        Ok(self)
    }

    pub fn close_contour(mut self) -> Result<Self, CurveBuilderError> {
        let Some(path) = self.current.as_mut() else {
            return Err(CurveError::MissingMoveTo.into());
        };

        if path.segments.is_empty() {
            return Err(CurveError::EmptyPath.into());
        }

        if let Some(end_point) = path.end_point()
            && end_point != path.start
        {
            path.segments.push(CurveSegment::Line { to: path.start });
        }

        self.flush_current()?;
        Ok(self)
    }

    pub fn build_shape(mut self) -> Result<CurveShape<I>, CurveBuilderError> {
        self.flush_current()?;

        if self.paths.is_empty() {
            Err(CurveError::NoContours.into())
        } else {
            Ok(CurveShape { contours: self.paths })
        }
    }

    pub fn build_path(mut self) -> Result<CurvePath<I>, CurveBuilderError> {
        if let Some(path) = self.current.take()
            && !path.segments.is_empty()
        {
            return Ok(path);
        }
        Err(CurveError::EmptyPath.into())
    }

    fn push_segment(&mut self, segment: CurveSegment<I>) -> Result<(), CurveError> {
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

impl<I: IntNumber> CurvePath<I> {
    fn is_closed(&self) -> bool {
        self.end_point().is_some_and(|end_point| end_point == self.start)
    }

    fn end_point(&self) -> Option<IntPoint<I>> {
        self.segments.last().map(CurveSegment::end_point)
    }
}

impl<I: IntNumber> CurveSegment<I> {
    fn end_point(&self) -> IntPoint<I> {
        match self {
            Self::Line { to } | Self::Quad { to, .. } | Self::Cubic { to, .. } => *to,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::curve::segment::CurveSegment;
    use i_overlay::i_float::float::rect::FloatRect;

    fn builder() -> CurveBuilder<[f64; 2], i32> {
        let adapter = FloatPointAdapter::with_scale(FloatRect::new(-300.0, 300.0, -300.0, 300.0), 1.0);
        CurveBuilder::new(adapter)
    }

    #[test]
    fn build_shape_with_multiple_contours() -> Result<(), CurveBuilderError> {
        let shape = builder()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .quad_to([1.0, 1.0], [0.0, 1.0])?
            .close_contour()?
            .move_to([2.0, 2.0])?
            .cubic_to([3.0, 2.0], [3.0, 3.0], [2.0, 3.0])?
            .close_contour()?
            .build_shape()?;

        assert_eq!(shape.contours.len(), 2);
        assert_eq!(shape.contours[0].start, IntPoint::new(0, 0));
        assert_eq!(shape.contours[0].segments.len(), 3);
        assert_eq!(shape.contours[1].start, IntPoint::new(2, 2));
        assert_eq!(shape.contours[1].segments.len(), 2);

        Ok(())
    }

    #[test]
    fn build_closed_polygon_preserves_explicit_closing_endpoint() -> Result<(), CurveBuilderError> {
        let shape = builder()
            .move_to([-210.0, -130.0])?
            .line_to([70.0, -130.0])?
            .line_to([70.0, 130.0])?
            .line_to([-216.0, 130.0])?
            .line_to([-210.0, -130.0])?
            .build_shape()?;

        assert_eq!(shape.contours.len(), 1);
        let contour = &shape.contours[0];
        assert_eq!(contour.start, IntPoint::new(-210, -130));
        match contour.segments.last() {
            Some(CurveSegment::Line { to }) => assert_eq!(*to, contour.start),
            _ => panic!("expected closing line segment"),
        }

        Ok(())
    }

    #[test]
    fn segment_without_move_to_is_error() {
        let result = builder().line_to([1.0, 0.0]);

        assert!(matches!(
            result,
            Err(CurveBuilderError::Curve(CurveError::MissingMoveTo))
        ));
    }

    #[test]
    fn close_with_line_closes_open_contour() -> Result<(), CurveBuilderError> {
        let shape = builder()
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
    fn build_requires_closed_contour() -> Result<(), CurveBuilderError> {
        let result = builder().move_to([0.0, 0.0])?.line_to([1.0, 0.0])?.build_shape();

        assert!(matches!(
            result,
            Err(CurveBuilderError::Curve(CurveError::UnclosedContour))
        ));
        Ok(())
    }

    #[test]
    fn empty_contour_is_error() -> Result<(), CurveBuilderError> {
        let result = builder().move_to([0.0, 0.0])?.build_shape();

        assert!(matches!(
            result,
            Err(CurveBuilderError::Curve(CurveError::EmptyPath))
        ));
        Ok(())
    }

    #[test]
    fn out_of_range_point_is_error() {
        let result = builder().move_to([400.0, 0.0]);

        assert!(matches!(
            result,
            Err(CurveBuilderError::Adapter(
                FloatPointAdapterRangeError::PointOutOfRange
            ))
        ));
    }
}
