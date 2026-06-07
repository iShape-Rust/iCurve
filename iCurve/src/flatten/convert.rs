use crate::curve::arc::EllipticArc;
use crate::curve::contour::CurveContour;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::cubic::{CubicSelfIntersection, find_cubic_self_intersection};
use crate::flatten::rect::ShapeFloatRect;
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, Segment,
};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;

pub trait ShapeToSegments<P: FloatPointCompatible> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<Segment<P>>;

    fn try_to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<Vec<Segment<P>>, FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible> ShapeToSegments<P> for CurveShape<P> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<Segment<P>> {
        let rect = self.float_rect().unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::<P, i32>::new(rect);
        self.try_to_normalize_segments_with_adapter(shape_type, &adapter)
            .expect("adapter rect must contain all curve points")
    }

    fn try_to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<Vec<Segment<P>>, FloatPointAdapterRangeError> {
        let mut result = Vec::with_capacity(self.segments_count());

        for contour in &self.contours {
            contour.try_extend_normalize_segments_with_adapter(shape_type, adapter, &mut result)?;
        }

        Ok(result)
    }
}

impl<P: FloatPointCompatible> ShapeToSegments<P> for CurveContour<P> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<Segment<P>> {
        let rect = self.float_rect().unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::<P, i32>::new(rect);
        self.try_to_normalize_segments_with_adapter(shape_type, &adapter)
            .expect("adapter rect must contain all curve points")
    }

    fn try_to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<Vec<Segment<P>>, FloatPointAdapterRangeError> {
        let mut result = Vec::with_capacity(self.segments_count());
        self.try_extend_normalize_segments_with_adapter(shape_type, adapter, &mut result)?;
        Ok(result)
    }
}

trait ContourToSegments<P: FloatPointCompatible> {
    fn try_extend_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<Segment<P>>,
    ) -> Result<(), FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible> ContourToSegments<P> for CurveContour<P> {
    fn try_extend_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<Segment<P>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let mut point = self.start;
        let mut buffer = Vec::with_capacity(8);

        for curve_segment in &self.segments {
            let next = curve_segment.try_normalize(point, adapter, &mut buffer)?;
            for normalized_segment in buffer.drain(..) {
                output.push(Segment {
                    normalized_segment,
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
        p0: P,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
    ) -> Result<P, FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible, I: IntNumber> CurveSegmentNormalization<P, I> for CurveSegment<P> {
    fn try_normalize(
        &self,
        p0: P,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
    ) -> Result<P, FloatPointAdapterRangeError> {
        match *self {
            CurveSegment::Line { to } => {
                if let Some(segment) = NormalizedSegment::try_line_with_adapter(p0, to, adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Quad { ctrl, to } => {
                if let Some(segment) = NormalizedSegment::try_quad_with_adapter(p0, ctrl, to, adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                match NormalizedSegment::try_cubic_with_adapter(p0, ctrl0, ctrl1, to, adapter)? {
                    TryCubicResult::None => {}
                    TryCubicResult::Segment(segment) => output.push(segment),
                    TryCubicResult::Segments(mut segments) => output.append(&mut segments),
                }
                Ok(to)
            }
            CurveSegment::Arc { ref arc } => {
                let p1 = arc.end_point();
                if let Some(segment) = NormalizedSegment::try_arc_with_adapter(p0, arc, adapter)? {
                    output.push(segment);
                };
                Ok(p1)
            }
        }
    }
}

impl<P: FloatPointCompatible> NormalizedSegment<P> {
    fn try_line_with_adapter<I: IntNumber>(
        p0: P,
        p1: P,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<Option<Self>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        if q0 != q1 {
            Ok(Some(NormalizedSegment::Line(LineSegment {
                control_points: [p0, p1],
            })))
        } else {
            Ok(None)
        }
    }

    fn try_quad_with_adapter<I: IntNumber>(
        p0: P,
        p1: P,
        p2: P,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<Option<Self>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        if q0 == q1 && q1 == q2 {
            Ok(None)
        } else if q0 != q2 && Triangle::is_line(q0, q1, q2) {
            Self::try_line_with_adapter(p0, p2, adapter)
        } else {
            Ok(Some(NormalizedSegment::Quad(QuadSegment {
                control_points: [p0, p1, p2],
            })))
        }
    }

    fn try_cubic_with_adapter<I: IntNumber>(
        p0: P,
        p1: P,
        p2: P,
        p3: P,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<TryCubicResult<P>, FloatPointAdapterRangeError> {
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
            let [first, last] = cubic.split_at_half();
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

        Ok(TryCubicResult::Segment(NormalizedSegment::Cubic(CubicSegment {
            control_points: [p0, p1, p2, p3],
        })))
    }

    fn split_self_intersecting_cubic_with_adapter<I: IntNumber>(
        p0: P,
        p1: P,
        p2: P,
        p3: P,
        intersection: CubicSelfIntersection<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<Vec<NormalizedSegment<P>>, FloatPointAdapterRangeError> {
        let (t0, t1) = if intersection.t0 < intersection.t1 {
            (intersection.t0, intersection.t1)
        } else {
            (intersection.t1, intersection.t0)
        };

        let cubic = CubicSegment {
            control_points: [p0, p1, p2, p3],
        };
        let [mut first, rest] = cubic.split_at(t0);
        let t = (t1 - t0) / (P::Scalar::from_float(1.0) - t0);
        let [mut middle, mut last] = rest.split_at(t);

        let point = intersection.point;
        first.control_points[3] = point;
        middle.control_points[0] = point;
        middle.control_points[3] = point;
        last.control_points[0] = point;

        let [middle_0, middle_1] = middle.split_at_half();

        let mut segments = Vec::with_capacity(6);
        Self::push_split_cubic_part(first, adapter, &mut segments)?;
        Self::push_split_cubic_part(middle_0, adapter, &mut segments)?;
        Self::push_split_cubic_part(middle_1, adapter, &mut segments)?;
        Self::push_split_cubic_part(last, adapter, &mut segments)?;
        Ok(segments)
    }

    fn push_split_cubic_part<I: IntNumber>(
        cubic: CubicSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let [p0, _, _, p3] = cubic.control_points;
        if adapter.try_float_to_int(&p0)? == adapter.try_float_to_int(&p3)? {
            let [first, last] = cubic.split_at_half();
            Self::push_cubic_without_self_intersection(first, adapter, output)?;
            Self::push_cubic_without_self_intersection(last, adapter, output)?;
        } else {
            Self::push_cubic_without_self_intersection(cubic, adapter, output)?;
        }

        Ok(())
    }

    fn push_cubic_without_self_intersection<I: IntNumber>(
        cubic: CubicSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
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
            output.push(NormalizedSegment::Cubic(cubic));
        }

        Ok(())
    }

    fn try_arc_with_adapter<I: IntNumber>(
        p0: P,
        arc: &EllipticArc<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<Option<Self>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let p1 = arc.end_point();
        let q1 = adapter.try_float_to_int(&p1)?;
        let center = adapter.try_float_to_int(&arc.center)?;
        if q0 == q1 || center == q0 || center == q1 {
            return Ok(None);
        }

        Ok(Some(NormalizedSegment::Arc(ArcSegment {
            p0,
            p1,
            center: arc.center,
            radii: arc.radii,
            rotation: arc.rotation,
            start_angle: arc.start_angle,
            sweep_angle: arc.sweep_angle,
        })))
    }
}

enum TryCubicResult<P: FloatPointCompatible> {
    None,
    Segment(NormalizedSegment<P>),
    Segments(Vec<NormalizedSegment<P>>),
}

impl<P: FloatPointCompatible> From<Option<NormalizedSegment<P>>> for TryCubicResult<P> {
    fn from(segment: Option<NormalizedSegment<P>>) -> Self {
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

impl<P: FloatPointCompatible> ShapeSegmentCount for CurveContour<P> {
    fn segments_count(&self) -> usize {
        self.segments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::builder::{CurveError, CurveShapeBuilder};
    use i_overlay::core::overlay::ShapeType;

    #[test]
    fn convert_shape_segments() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .quad_to([1.0, 1.0], [0.0, 1.0])?
            .cubic_to([-1.0, 1.0], [-1.0, 0.0], [0.0, 0.0])?
            .build()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 3);

        match &segments[0].normalized_segment {
            NormalizedSegment::Line(segment) => assert_eq!(segment.control_points, [[0.0, 0.0], [1.0, 0.0]]),
            _ => panic!("Expected line segment"),
        }

        match &segments[1].normalized_segment {
            NormalizedSegment::Quad(segment) => {
                assert_eq!(segment.control_points, [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
            }
            _ => panic!("Expected quad segment"),
        }

        match &segments[2].normalized_segment {
            NormalizedSegment::Cubic(segment) => assert_eq!(
                segment.control_points,
                [[0.0, 1.0], [-1.0, 1.0], [-1.0, 0.0], [0.0, 0.0]]
            ),
            _ => panic!("Expected cubic segment"),
        }

        Ok(())
    }

    #[test]
    fn convert_shape_segments_with_adapter() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([1.0, 0.0])?
            .close_with_line()?
            .build()?;
        let points = [[0.0, 0.0], [1.0, 0.0]];
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter());

        let segments = shape
            .try_to_normalize_segments_with_adapter(ShapeType::Subject, &adapter)
            .expect("points must fit adapter rect");

        assert_eq!(segments.len(), 2);
        match &segments[0].normalized_segment {
            NormalizedSegment::Line(segment) => {
                assert_eq!(segment.control_points, [[0.0, 0.0], [1.0, 0.0]])
            }
            _ => panic!("Expected line segment"),
        }

        Ok(())
    }

    #[test]
    fn convert_closed_polygon_preserves_normalized_closing_endpoint() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([-210.0_f32, -130.0])?
            .line_to([70.0, -130.0])?
            .line_to([70.0, 130.0])?
            .line_to([-216.049_59, 129.983_02])?
            .line_to([-210.0, -130.0])?
            .build()?;
        let clip_bounds = [[-70.0_f32, -170.0], [210.0, 90.0]];
        let adapter = FloatPointAdapter::<[f32; 2], i32>::with_iter(
            shape
                .contours
                .iter()
                .flat_map(|contour| {
                    core::iter::once(&contour.start).chain(contour.segments.iter().filter_map(|segment| {
                        match segment {
                            crate::curve::segment::CurveSegment::Line { to } => Some(to),
                            _ => None,
                        }
                    }))
                })
                .chain(clip_bounds.iter()),
        );

        let segments = shape
            .try_to_normalize_segments_with_adapter(ShapeType::Subject, &adapter)
            .expect("points must fit adapter rect");

        assert_eq!(segments.len(), 4);
        let last = match &segments[3].normalized_segment {
            NormalizedSegment::Line(segment) => segment,
            _ => panic!("expected closing line segment"),
        };

        assert_eq!(last.control_points[1], shape.contours[0].start);
        assert_eq!(
            adapter.float_to_int(&last.control_points[1]),
            adapter.float_to_int(&shape.contours[0].start)
        );

        Ok(())
    }

    #[test]
    fn convert_segments_with_adapter_returns_range_error() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .line_to([2.0, 0.0])?
            .close_with_line()?
            .build()?;
        let points = [[0.0, 0.0], [1.0, 0.0]];
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter());

        let error = shape
            .try_to_normalize_segments_with_adapter(ShapeType::Subject, &adapter)
            .err()
            .expect("point outside adapter rect must return an error");

        assert_eq!(error, FloatPointAdapterRangeError::PointOutOfRange);

        Ok(())
    }

    #[test]
    fn convert_self_intersecting_cubic_segments() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .cubic_to([-3.0, -3.0], [-3.0, -2.0], [-2.0, -2.0])?
            .close_with_line()?
            .build()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 5);
        let point = [-2.3615160349854225, -2.0466472303206995];
        match (
            &segments[0].normalized_segment,
            &segments[1].normalized_segment,
            &segments[2].normalized_segment,
            &segments[3].normalized_segment,
        ) {
            (
                NormalizedSegment::Cubic(first),
                NormalizedSegment::Cubic(middle_0),
                NormalizedSegment::Cubic(middle_1),
                NormalizedSegment::Cubic(last),
            ) => {
                assert_point_eq(first.control_points[3], point);
                assert_point_eq(middle_0.control_points[0], point);
                assert_point_eq(middle_1.control_points[3], point);
                assert_point_eq(last.control_points[0], point);
            }
            _ => panic!("Expected cubic segments"),
        }
        match &segments[4].normalized_segment {
            NormalizedSegment::Line(segment) => {
                assert_eq!(segment.control_points, [[-2.0, -2.0], [0.0, 0.0]])
            }
            _ => panic!("Expected closing line segment"),
        }

        Ok(())
    }

    #[test]
    fn convert_closed_cubic_splits_at_half() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .cubic_to([1.0, 2.0], [-1.0, 2.0], [0.0, 0.0])?
            .build()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 2);
        match (&segments[0].normalized_segment, &segments[1].normalized_segment) {
            (NormalizedSegment::Cubic(first), NormalizedSegment::Cubic(last)) => {
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
        let p0 = [0.0, 0.0];
        let p1 = [-3.0, -3.0];
        let p2 = [-3.0, -2.0];
        let p3 = [-2.0, -2.0];
        let points = [p0, p1, p2, p3];
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter());
        let intersection = CubicSelfIntersection {
            t0: 3.0 / 7.0,
            t1: 6.0 / 7.0,
            point: p0,
        };

        let segments = NormalizedSegment::split_self_intersecting_cubic_with_adapter(
            p0,
            p1,
            p2,
            p3,
            intersection,
            &adapter,
        )
        .expect("points must fit adapter rect");

        assert!(!segments.is_empty());
        for segment in segments {
            if let NormalizedSegment::Cubic(segment) = segment {
                let q0 = adapter.float_to_int(&segment.control_points[0]);
                let q1 = adapter.float_to_int(&segment.control_points[3]);
                assert_ne!(q0, q1);
            }
        }
    }

    #[test]
    fn convert_arc_endpoint() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([1.0, 0.0])?
            .arc_to(EllipticArc {
                center: [0.0, 0.0],
                radii: [1.0, 1.0],
                rotation: 0.0,
                start_angle: 0.0,
                sweep_angle: core::f64::consts::FRAC_PI_2,
            })?
            .close_with_line()?
            .build()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        match &segments[0].normalized_segment {
            NormalizedSegment::Arc(segment) => {
                assert_eq!(segment.p0, [1.0, 0.0]);
                assert!(segment.p1[0].abs() < 0.000001);
                assert!((segment.p1[1] - 1.0).abs() < 0.000001);
            }
            _ => panic!("Expected arc segment"),
        }

        Ok(())
    }

    fn assert_point_eq(a: [f64; 2], b: [f64; 2]) {
        assert!((a[0] - b[0]).abs() < 0.000001);
        assert!((a[1] - b[1]).abs() < 0.000001);
    }
}
