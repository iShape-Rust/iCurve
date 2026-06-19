use crate::curve::path::CurvePath;
use crate::curve::rect::CurveToFloatRect;
use crate::curve::shape::CurveShape;
use crate::kernel::curve::segment::Segment;
use crate::normalization::segment::CurveSegmentNormalization;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;

pub trait CurveToSegments<T: FloatNumber> {
    fn try_to_normalize_segments<I: IntNumber>(&self) -> Result<Vec<Segment<T>>, FloatPointAdapterRangeError>
    where
        Self: CurveToFloatRect<T>,
    {
        let rect = self.float_rect().unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::<_, I>::new(rect);
        let mut result = Vec::new();
        self.try_extend_normalize_segments_with_adapter(&adapter, &mut result)?;
        Ok(result)
    }

    fn try_to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Vec<Segment<T>>, FloatPointAdapterRangeError> {
        let mut result = Vec::new();
        self.try_extend_normalize_segments_with_adapter(adapter, &mut result)?;
        Ok(result)
    }

    fn try_extend_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        output: &mut Vec<Segment<T>>,
    ) -> Result<(), FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible<Scalar = T>, T: FloatNumber> CurveToSegments<T> for CurvePath<P> {
    fn try_extend_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        output: &mut Vec<Segment<T>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let mut point = self.start;

        for curve_segment in &self.segments {
            let next = curve_segment.try_normalize(FloatPoint::from_point(point), adapter, output)?;

            point = next;
        }

        Ok(())
    }
}

impl<P: FloatPointCompatible<Scalar = T>, T: FloatNumber> CurveToSegments<T> for CurveShape<P> {
    fn try_extend_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        output: &mut Vec<Segment<T>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let init_capacity = self
            .contours
            .iter()
            .fold(0, |count, contour| count + contour.segments.len());

        output.reserve(init_capacity);

        for path in self.contours.iter() {
            path.try_extend_normalize_segments_with_adapter(adapter, output)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::builder::{CurveBuilder, CurveError};
    use crate::kernel::curve::segment::Segment;

    #[test]
    fn normalize_shape_segments() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .quad_to([1.0, 1.0], [0.0, 1.0])?
            .cubic_to([-1.0, 1.0], [-1.0, 0.0], [0.0, 0.0])?
            .build_shape()?;

        let segments = shape.try_to_normalize_segments::<i32>().unwrap();

        assert_eq!(segments.len(), 3);

        match &segments[0] {
            Segment::Line(segment) => {
                assert_control_points_eq(segment.control_points, [[0.0, 0.0], [1.0, 0.0]])
            }
            _ => panic!("Expected line segment"),
        }

        match &segments[1] {
            Segment::Quad(segment) => {
                assert_control_points_eq(segment.control_points, [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
            }
            _ => panic!("Expected quad segment"),
        }

        match &segments[2] {
            Segment::Cubic(segment) => assert_control_points_eq(
                segment.control_points,
                [[0.0, 1.0], [-1.0, 1.0], [-1.0, 0.0], [0.0, 0.0]],
            ),
            _ => panic!("Expected cubic segment"),
        }

        Ok(())
    }

    #[test]
    fn normalize_shape_segments_with_adapter() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .close_contour()?
            .build_shape()?;
        let points = [FloatPoint::new(0.0, 0.0), FloatPoint::new(1.0, 0.0)];
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(points.iter());

        let segments = shape
            .try_to_normalize_segments_with_adapter(&adapter)
            .expect("points must fit adapter rect");

        assert_eq!(segments.len(), 2);
        match &segments[0] {
            Segment::Line(segment) => {
                assert_control_points_eq(segment.control_points, [[0.0, 0.0], [1.0, 0.0]])
            }
            _ => panic!("Expected line segment"),
        }

        Ok(())
    }

    #[test]
    fn normalize_closed_polygon_preserves_normalized_closing_endpoint() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([-210.0_f32, -130.0])?
            .line_to([70.0, -130.0])?
            .line_to([70.0, 130.0])?
            .line_to([-216.049_59, 129.983_02])?
            .line_to([-210.0, -130.0])?
            .build_shape()?;
        let clip_bounds = [[-70.0_f32, -170.0], [210.0, 90.0]];
        let points: Vec<_> = shape
            .contours
            .iter()
            .flat_map(|contour| {
                core::iter::once(contour.start).chain(contour.segments.iter().filter_map(|segment| {
                    match segment {
                        crate::curve::segment::CurveSegment::Line { to } => Some(*to),
                        _ => None,
                    }
                }))
            })
            .chain(clip_bounds)
            .map(FloatPoint::from_point)
            .collect();
        let adapter = FloatPointAdapter::<FloatPoint<f32>, i32>::with_iter(points.iter());

        let segments = shape
            .try_to_normalize_segments_with_adapter(&adapter)
            .expect("points must fit adapter rect");

        assert_eq!(segments.len(), 4);
        let last = match &segments[3] {
            Segment::Line(segment) => segment,
            _ => panic!("expected closing line segment"),
        };

        assert_point_eq(last.control_points[1], shape.contours[0].start);
        assert_eq!(
            adapter.float_to_int(&last.control_points[1]),
            adapter.float_to_int(&FloatPoint::from_point(shape.contours[0].start))
        );

        Ok(())
    }

    #[test]
    fn normalize_segments_with_adapter_returns_range_error() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([2.0, 0.0])?
            .close_contour()?
            .build_shape()?;
        let points = [FloatPoint::new(0.0, 0.0), FloatPoint::new(1.0, 0.0)];
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(points.iter());

        let error = shape
            .try_to_normalize_segments_with_adapter(&adapter)
            .err()
            .expect("point outside adapter rect must return an error");

        assert_eq!(error, FloatPointAdapterRangeError::PointOutOfRange);

        Ok(())
    }

    #[test]
    fn normalize_self_intersecting_cubic_segments() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .cubic_to([-3.0, -3.0], [-3.0, -2.0], [-2.0, -2.0])?
            .close_contour()?
            .build_shape()?;

        let segments = shape.try_to_normalize_segments::<i32>().unwrap();

        assert_eq!(segments.len(), 5);
        let point = [-2.3615160349854225, -2.0466472303206995];
        match (&segments[0], &segments[1], &segments[2], &segments[3]) {
            (
                Segment::Cubic(first),
                Segment::Cubic(middle_0),
                Segment::Cubic(middle_1),
                Segment::Cubic(last),
            ) => {
                assert_point_eq(first.control_points[3], point);
                assert_point_eq(middle_0.control_points[0], point);
                assert_point_eq(middle_1.control_points[3], point);
                assert_point_eq(last.control_points[0], point);
            }
            _ => panic!("Expected cubic segments"),
        }
        match &segments[4] {
            Segment::Line(segment) => {
                assert_control_points_eq(segment.control_points, [[-2.0, -2.0], [0.0, 0.0]])
            }
            _ => panic!("Expected closing line segment"),
        }

        Ok(())
    }

    #[test]
    fn normalize_closed_cubic_splits_at_half() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .cubic_to([1.0, 2.0], [-1.0, 2.0], [0.0, 0.0])?
            .build_shape()?;

        let segments = shape.try_to_normalize_segments::<i32>().unwrap();

        assert_eq!(segments.len(), 2);
        match (&segments[0], &segments[1]) {
            (Segment::Cubic(first), Segment::Cubic(last)) => {
                assert_point_eq(first.control_points[0], [0.0, 0.0]);
                assert_point_eq(first.control_points[3], [0.0, 1.5]);
                assert_point_eq(last.control_points[0], [0.0, 1.5]);
                assert_point_eq(last.control_points[3], [0.0, 0.0]);
            }
            _ => panic!("Expected cubic segments"),
        }

        Ok(())
    }

    fn assert_control_points_eq<T: FloatNumber, const N: usize>(
        actual: [FloatPoint<T>; N],
        expected: [[T; 2]; N],
    ) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert_point_eq(actual, expected);
        }
    }

    fn assert_point_eq<T: FloatNumber>(a: FloatPoint<T>, b: [T; 2]) {
        assert!((a.x.to_f64() - b[0].to_f64()).abs() < 0.000001);
        assert!((a.y.to_f64() - b[1].to_f64()).abs() < 0.000001);
    }
}
