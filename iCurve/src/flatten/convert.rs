use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::cubic::{CubicSelfIntersection, find_cubic_self_intersection};
use crate::flatten::rect::ShapeFloatRect;
use crate::flatten::segment::ShapeSegment;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;
use crate::kernel::curve::split_at::SplitAt;
use alloc::vec::Vec;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;
use crate::curve::path::CurvePath;

pub trait ShapeToSegments<P: FloatPointCompatible> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<ShapeSegment<P::Scalar>>;

    fn try_to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
    ) -> Result<Vec<ShapeSegment<P::Scalar>>, FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible> ShapeToSegments<P> for CurveShape<P> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<ShapeSegment<P::Scalar>> {
        let rect = self.float_rect().unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::<FloatPoint<P::Scalar>, i32>::new(rect);
        self.try_to_normalize_segments_with_adapter(shape_type, &adapter)
            .expect("adapter rect must contain all curve points")
    }

    fn try_to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
    ) -> Result<Vec<ShapeSegment<P::Scalar>>, FloatPointAdapterRangeError> {
        let mut result = Vec::with_capacity(self.segments_count());

        for contour in &self.contours {
            contour.try_extend_normalize_segments_with_adapter(shape_type, adapter, &mut result)?;
        }

        Ok(result)
    }
}

impl<P: FloatPointCompatible> ShapeToSegments<P> for CurvePath<P> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<ShapeSegment<P::Scalar>> {
        let rect = self.float_rect().unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::<FloatPoint<P::Scalar>, i32>::new(rect);
        self.try_to_normalize_segments_with_adapter(shape_type, &adapter)
            .expect("adapter rect must contain all curve points")
    }

    fn try_to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
    ) -> Result<Vec<ShapeSegment<P::Scalar>>, FloatPointAdapterRangeError> {
        let mut result = Vec::with_capacity(self.segments_count());
        self.try_extend_normalize_segments_with_adapter(shape_type, adapter, &mut result)?;
        Ok(result)
    }
}

trait ContourToSegments<P: FloatPointCompatible> {
    fn try_extend_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<ShapeSegment<P::Scalar>>,
    ) -> Result<(), FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible> ContourToSegments<P> for CurvePath<P> {
    fn try_extend_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<ShapeSegment<P::Scalar>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let mut point = self.start;
        let mut buffer = Vec::with_capacity(8);

        for curve_segment in &self.segments {
            let next = curve_segment.try_normalize(FloatPoint::from_point(point), adapter, &mut buffer)?;
            for normalized_segment in buffer.drain(..) {
                output.push(ShapeSegment {
                    segment: normalized_segment,
                    shape_type,
                });
            }

            point = next;
        }

        Ok(())
    }
}

trait CurveSegmentNormalization<P: FloatPointCompatible, I: IntNumber> {
    fn try_normalize(
        &self,
        p0: FloatPoint<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<Segment<P::Scalar>>,
    ) -> Result<P, FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible, I: IntNumber> CurveSegmentNormalization<P, I> for CurveSegment<P> {
    fn try_normalize(
        &self,
        p0: FloatPoint<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<Segment<P::Scalar>>,
    ) -> Result<P, FloatPointAdapterRangeError> {
        match *self {
            CurveSegment::Line { to } => {
                let p1 = FloatPoint::from_point(to);
                if let Some(segment) = Segment::try_line_with_adapter(p0, p1, adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Quad { ctrl, to } => {
                let p1 = FloatPoint::from_point(ctrl);
                let p2 = FloatPoint::from_point(to);
                if let Some(segment) = Segment::try_quad_with_adapter(p0, p1, p2, adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                let p1 = FloatPoint::from_point(ctrl0);
                let p2 = FloatPoint::from_point(ctrl1);
                let p3 = FloatPoint::from_point(to);
                match Segment::try_cubic_with_adapter(p0, p1, p2, p3, adapter)? {
                    TryCubicResult::None => {}
                    TryCubicResult::Segment(segment) => output.push(segment),
                    TryCubicResult::Segments(mut segments) => output.append(&mut segments),
                }
                Ok(to)
            }
        }
    }
}

impl<T: FloatNumber> Segment<T> {
    fn try_line_with_adapter<I: IntNumber>(
        p0: FloatPoint<T>,
        p1: FloatPoint<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Option<Self>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        if q0 != q1 {
            Ok(Some(Segment::Line(LineSegment {
                control_points: [p0, p1],
            })))
        } else {
            Ok(None)
        }
    }

    fn try_quad_with_adapter<I: IntNumber>(
        p0: FloatPoint<T>,
        p1: FloatPoint<T>,
        p2: FloatPoint<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Option<Self>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        if q0 == q1 && q1 == q2 {
            Ok(None)
        } else if q0 != q2 && Triangle::is_line(q0, q1, q2) {
            Self::try_line_with_adapter(p0, p2, adapter)
        } else {
            Ok(Some(Segment::Quad(QuadSegment {
                control_points: [p0, p1, p2],
            })))
        }
    }

    fn try_cubic_with_adapter<I: IntNumber>(
        p0: FloatPoint<T>,
        p1: FloatPoint<T>,
        p2: FloatPoint<T>,
        p3: FloatPoint<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<TryCubicResult<T>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        let q3 = adapter.try_float_to_int(&p3)?;
        if q0 == q1 && q1 == q2 && q2 == q3 {
            return Ok(TryCubicResult::None);
        }

        if q0 == q3 {
            let cubic = CubicSegment {
                control_points: [p0, p1, p2, p3],
            };
            let [first, last] = cubic.split_at(T::HALF);
            let mut segments = Vec::with_capacity(2);
            Self::push_cubic_without_self_intersection(first, adapter, &mut segments)?;
            Self::push_cubic_without_self_intersection(last, adapter, &mut segments)?;
            return if segments.is_empty() {
                Ok(TryCubicResult::None)
            } else {
                Ok(TryCubicResult::Segments(segments))
            };
        }

        if q1 == q2 {
            return Ok(Self::try_quad_with_adapter(p0, p1, p3, adapter)?.into());
        }

        if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            return Ok(Self::try_line_with_adapter(p0, p3, adapter)?.into());
        }

        if let Some(intersection) = find_cubic_self_intersection(p0, p1, p2, p3) {
            let segments =
                Self::split_self_intersecting_cubic_with_adapter(p0, p1, p2, p3, intersection, adapter)?;
            return if segments.is_empty() {
                Ok(TryCubicResult::None)
            } else {
                Ok(TryCubicResult::Segments(segments))
            };
        }

        Ok(TryCubicResult::Segment(Segment::Cubic(CubicSegment {
            control_points: [p0, p1, p2, p3],
        })))
    }

    fn split_self_intersecting_cubic_with_adapter<I: IntNumber>(
        p0: FloatPoint<T>,
        p1: FloatPoint<T>,
        p2: FloatPoint<T>,
        p3: FloatPoint<T>,
        intersection: CubicSelfIntersection<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Vec<Segment<T>>, FloatPointAdapterRangeError> {
        let (t0, t1) = if intersection.t0 < intersection.t1 {
            (intersection.t0, intersection.t1)
        } else {
            (intersection.t1, intersection.t0)
        };

        let cubic = CubicSegment {
            control_points: [p0, p1, p2, p3],
        };
        let [mut first, rest] = cubic.split_at(t0);
        let t = (t1 - t0) / (T::ONE - t0);
        let [mut middle, mut last] = rest.split_at(t);

        let point = intersection.point;
        first.control_points[3] = point;
        middle.control_points[0] = point;
        middle.control_points[3] = point;
        last.control_points[0] = point;

        let [middle_0, middle_1] = middle.split_at(T::HALF);

        let mut segments = Vec::with_capacity(6);
        Self::push_split_cubic_part(first, adapter, &mut segments)?;
        Self::push_split_cubic_part(middle_0, adapter, &mut segments)?;
        Self::push_split_cubic_part(middle_1, adapter, &mut segments)?;
        Self::push_split_cubic_part(last, adapter, &mut segments)?;
        Ok(segments)
    }

    fn push_split_cubic_part<I: IntNumber>(
        cubic: CubicSegment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        output: &mut Vec<Segment<T>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let [p0, _, _, p3] = cubic.control_points;
        if adapter.try_float_to_int(&p0)? == adapter.try_float_to_int(&p3)? {
            let [first, last] = cubic.split_at(T::HALF);
            Self::push_cubic_without_self_intersection(first, adapter, output)?;
            Self::push_cubic_without_self_intersection(last, adapter, output)?;
        } else {
            Self::push_cubic_without_self_intersection(cubic, adapter, output)?;
        }

        Ok(())
    }

    fn push_cubic_without_self_intersection<I: IntNumber>(
        cubic: CubicSegment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        output: &mut Vec<Segment<T>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let [p0, p1, p2, p3] = cubic.control_points;
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        let q3 = adapter.try_float_to_int(&p3)?;
        if q0 == q1 && q1 == q2 && q2 == q3 {
            return Ok(());
        }

        if q1 == q2 {
            if let Some(segment) = Self::try_quad_with_adapter(p0, p1, p3, adapter)? {
                output.push(segment);
            }
        } else if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            if let Some(segment) = Self::try_line_with_adapter(p0, p3, adapter)? {
                output.push(segment);
            }
        } else {
            output.push(Segment::Cubic(cubic));
        }

        Ok(())
    }
}

enum TryCubicResult<T: FloatNumber> {
    None,
    Segment(Segment<T>),
    Segments(Vec<Segment<T>>),
}

impl<T: FloatNumber> From<Option<Segment<T>>> for TryCubicResult<T> {
    fn from(segment: Option<Segment<T>>) -> Self {
        match segment {
            Some(segment) => Self::Segment(segment),
            None => Self::None,
        }
    }
}

trait ShapeSegmentCount {
    fn segments_count(&self) -> usize;
}

impl<P: FloatPointCompatible> ShapeSegmentCount for CurveShape<P> {
    fn segments_count(&self) -> usize {
        self.contours
            .iter()
            .fold(0, |count, contour| count + contour.segments.len())
    }
}

impl<P: FloatPointCompatible> ShapeSegmentCount for CurvePath<P> {
    fn segments_count(&self) -> usize {
        self.segments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::builder::{CurveError, CurveBuilder};
    use i_overlay::core::overlay::ShapeType;

    #[test]
    fn convert_shape_segments() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .quad_to([1.0, 1.0], [0.0, 1.0])?
            .cubic_to([-1.0, 1.0], [-1.0, 0.0], [0.0, 0.0])?
            .build_shape()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 3);

        match &segments[0].segment {
            Segment::Line(segment) => {
                assert_control_points_eq(segment.control_points, [[0.0, 0.0], [1.0, 0.0]])
            }
            _ => panic!("Expected line segment"),
        }

        match &segments[1].segment {
            Segment::Quad(segment) => {
                assert_control_points_eq(segment.control_points, [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
            }
            _ => panic!("Expected quad segment"),
        }

        match &segments[2].segment {
            Segment::Cubic(segment) => assert_control_points_eq(
                segment.control_points,
                [[0.0, 1.0], [-1.0, 1.0], [-1.0, 0.0], [0.0, 0.0]],
            ),
            _ => panic!("Expected cubic segment"),
        }

        Ok(())
    }

    #[test]
    fn convert_shape_segments_with_adapter() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .close_contour()?
            .build_shape()?;
        let points = [FloatPoint::new(0.0, 0.0), FloatPoint::new(1.0, 0.0)];
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(points.iter());

        let segments = shape
            .try_to_normalize_segments_with_adapter(ShapeType::Subject, &adapter)
            .expect("points must fit adapter rect");

        assert_eq!(segments.len(), 2);
        match &segments[0].segment {
            Segment::Line(segment) => {
                assert_control_points_eq(segment.control_points, [[0.0, 0.0], [1.0, 0.0]])
            }
            _ => panic!("Expected line segment"),
        }

        Ok(())
    }

    #[test]
    fn convert_closed_polygon_preserves_normalized_closing_endpoint() -> Result<(), CurveError> {
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
            .try_to_normalize_segments_with_adapter(ShapeType::Subject, &adapter)
            .expect("points must fit adapter rect");

        assert_eq!(segments.len(), 4);
        let last = match &segments[3].segment {
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
    fn convert_segments_with_adapter_returns_range_error() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([2.0, 0.0])?
            .close_contour()?
            .build_shape()?;
        let points = [FloatPoint::new(0.0, 0.0), FloatPoint::new(1.0, 0.0)];
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(points.iter());

        let error = shape
            .try_to_normalize_segments_with_adapter(ShapeType::Subject, &adapter)
            .err()
            .expect("point outside adapter rect must return an error");

        assert_eq!(error, FloatPointAdapterRangeError::PointOutOfRange);

        Ok(())
    }

    #[test]
    fn convert_self_intersecting_cubic_segments() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .cubic_to([-3.0, -3.0], [-3.0, -2.0], [-2.0, -2.0])?
            .close_contour()?
            .build_shape()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 5);
        let point = [-2.3615160349854225, -2.0466472303206995];
        match (
            &segments[0].segment,
            &segments[1].segment,
            &segments[2].segment,
            &segments[3].segment,
        ) {
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
        match &segments[4].segment {
            Segment::Line(segment) => {
                assert_control_points_eq(segment.control_points, [[-2.0, -2.0], [0.0, 0.0]])
            }
            _ => panic!("Expected closing line segment"),
        }

        Ok(())
    }

    #[test]
    fn convert_closed_cubic_splits_at_half() -> Result<(), CurveError> {
        let shape = CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .cubic_to([1.0, 2.0], [-1.0, 2.0], [0.0, 0.0])?
            .build_shape()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 2);
        match (&segments[0].segment, &segments[1].segment) {
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

    #[test]
    fn split_self_intersecting_cubic_handles_endpoint_intersection() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(-3.0, -3.0);
        let p2 = FloatPoint::new(-3.0, -2.0);
        let p3 = FloatPoint::new(-2.0, -2.0);
        let points = [p0, p1, p2, p3];
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(points.iter());
        let intersection = CubicSelfIntersection {
            t0: 3.0 / 7.0,
            t1: 6.0 / 7.0,
            point: p0,
        };

        let segments =
            Segment::split_self_intersecting_cubic_with_adapter(p0, p1, p2, p3, intersection, &adapter)
                .expect("points must fit adapter rect");

        assert!(!segments.is_empty());
        for segment in segments {
            if let Segment::Cubic(segment) = segment {
                let q0 = adapter.float_to_int(&segment.control_points[0]);
                let q1 = adapter.float_to_int(&segment.control_points[3]);
                assert_ne!(q0, q1);
            }
        }
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
