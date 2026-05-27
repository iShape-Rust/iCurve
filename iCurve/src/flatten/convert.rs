use crate::curve::arc::EllipticArc;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::cubic::{CubicSelfIntersection, find_cubic_self_intersection};
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, Segment,
};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;

pub(crate) trait ShapeToSegments<P: FloatPointCompatible> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<Segment<P>>;

    fn to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Vec<Segment<P>>;
}

impl<P: FloatPointCompatible> ShapeToSegments<P> for CurveShape<P> {
    fn to_normalize_segments(&self, shape_type: ShapeType) -> Vec<Segment<P>> {
        let rect = self.float_rect().unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::<P, i32>::new(rect);
        self.to_normalize_segments_with_adapter(shape_type, &adapter)
    }

    fn to_normalize_segments_with_adapter<I: IntNumber>(
        &self,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Vec<Segment<P>> {
        let mut result = Vec::with_capacity(self.segments_count());

        let mut buffer = Vec::with_capacity(8);

        for contour in &self.contours {
            let mut point = contour.start;

            for curve_segment in &contour.segments {
                let next = curve_segment.normalize(point, adapter, &mut buffer);
                for normalized_segment in buffer.drain(..) {
                    result.push(Segment {
                        normalized_segment,
                        shape_type,
                    });
                }

                point = next;
            }
        }

        result
    }
}

trait CurveSegmentNormalization<P: FloatPointCompatible, I: IntNumber> {
    fn normalize(
        &self,
        p0: P,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
    ) -> P;
}

impl<P: FloatPointCompatible, I: IntNumber> CurveSegmentNormalization<P, I> for CurveSegment<P> {
    fn normalize(
        &self,
        p0: P,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
    ) -> P {
        match *self {
            CurveSegment::Line { to } => {
                if let Some(segment) = NormalizedSegment::try_line_with_adapter(p0, to, adapter) {
                    output.push(segment);
                };
                to
            }
            CurveSegment::Quad { ctrl, to } => {
                if let Some(segment) = NormalizedSegment::try_quad_with_adapter(p0, ctrl, to, adapter) {
                    output.push(segment);
                };
                to
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                match NormalizedSegment::try_cubic_with_adapter(p0, ctrl0, ctrl1, to, adapter) {
                    TryCubicResult::None => {}
                    TryCubicResult::Segment(segment) => output.push(segment),
                    TryCubicResult::Segments(mut segments) => output.append(&mut segments),
                }
                to
            }
            CurveSegment::Arc { ref arc } => {
                let p1 = arc.end_point();
                if let Some(segment) = NormalizedSegment::try_arc_with_adapter(p0, arc, adapter) {
                    output.push(segment);
                };
                p1
            }
        }
    }
}

impl<P: FloatPointCompatible> NormalizedSegment<P> {
    fn try_line_with_adapter<I: IntNumber>(p0: P, p1: P, adapter: &FloatPointAdapter<P, I>) -> Option<Self> {
        let q0 = adapter.float_to_int(&p0);
        let q1 = adapter.float_to_int(&p1);
        if q0 != q1 {
            Some(NormalizedSegment::Line(LineSegment {
                control_points: [p0, p1],
            }))
        } else {
            None
        }
    }

    fn try_quad_with_adapter<I: IntNumber>(
        p0: P,
        p1: P,
        p2: P,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Option<Self> {
        let q0 = adapter.float_to_int(&p0);
        let q1 = adapter.float_to_int(&p1);
        let q2 = adapter.float_to_int(&p2);
        if q0 == q1 && q1 == q2 {
            None
        } else if q0 != q2 && Triangle::is_line(q0, q1, q2) {
            Self::try_line_with_adapter(p0, p2, adapter)
        } else {
            Some(NormalizedSegment::Quad(QuadSegment {
                control_points: [p0, p1, p2],
            }))
        }
    }

    fn try_cubic_with_adapter<I: IntNumber>(
        p0: P,
        p1: P,
        p2: P,
        p3: P,
        adapter: &FloatPointAdapter<P, I>,
    ) -> TryCubicResult<P> {
        let q0 = adapter.float_to_int(&p0);
        let q1 = adapter.float_to_int(&p1);
        let q2 = adapter.float_to_int(&p2);
        let q3 = adapter.float_to_int(&p3);
        if q0 == q1 && q1 == q2 && q2 == q3 {
            return TryCubicResult::None;
        }

        if q0 == q3 {
            let cubic = CubicSegment {
                control_points: [p0, p1, p2, p3],
            };
            let [first, last] = cubic.split_at_half();
            let mut segments = Vec::with_capacity(2);
            Self::push_cubic_without_self_intersection(first, adapter, &mut segments);
            Self::push_cubic_without_self_intersection(last, adapter, &mut segments);
            return if segments.is_empty() {
                TryCubicResult::None
            } else {
                TryCubicResult::Segments(segments)
            };
        }

        if q1 == q2 {
            return Self::try_quad_with_adapter(p0, p1, p3, adapter).into();
        }

        if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            return Self::try_line_with_adapter(p0, p3, adapter).into();
        }

        if let Some(intersection) = find_cubic_self_intersection(p0, p1, p2, p3) {
            let segments =
                Self::split_self_intersecting_cubic_with_adapter(p0, p1, p2, p3, intersection, adapter);
            return if segments.is_empty() {
                TryCubicResult::None
            } else {
                TryCubicResult::Segments(segments)
            };
        }

        TryCubicResult::Segment(NormalizedSegment::Cubic(CubicSegment {
            control_points: [p0, p1, p2, p3],
        }))
    }

    fn split_self_intersecting_cubic_with_adapter<I: IntNumber>(
        p0: P,
        p1: P,
        p2: P,
        p3: P,
        intersection: CubicSelfIntersection<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Vec<NormalizedSegment<P>> {
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
        Self::push_split_cubic_part(first, adapter, &mut segments);
        Self::push_split_cubic_part(middle_0, adapter, &mut segments);
        Self::push_split_cubic_part(middle_1, adapter, &mut segments);
        Self::push_split_cubic_part(last, adapter, &mut segments);
        segments
    }

    fn push_split_cubic_part<I: IntNumber>(
        cubic: CubicSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
    ) {
        let [p0, _, _, p3] = cubic.control_points;
        if adapter.float_to_int(&p0) == adapter.float_to_int(&p3) {
            let [first, last] = cubic.split_at_half();
            Self::push_cubic_without_self_intersection(first, adapter, output);
            Self::push_cubic_without_self_intersection(last, adapter, output);
        } else {
            Self::push_cubic_without_self_intersection(cubic, adapter, output);
        }
    }

    fn push_cubic_without_self_intersection<I: IntNumber>(
        cubic: CubicSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<NormalizedSegment<P>>,
    ) {
        let [p0, p1, p2, p3] = cubic.control_points;
        let q0 = adapter.float_to_int(&p0);
        let q1 = adapter.float_to_int(&p1);
        let q2 = adapter.float_to_int(&p2);
        let q3 = adapter.float_to_int(&p3);
        if q0 == q1 && q1 == q2 && q2 == q3 {
            return;
        }

        if q1 == q2 {
            if let Some(segment) = Self::try_quad_with_adapter(p0, p1, p3, adapter) {
                output.push(segment);
            }
        } else if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            if let Some(segment) = Self::try_line_with_adapter(p0, p3, adapter) {
                output.push(segment);
            }
        } else {
            output.push(NormalizedSegment::Cubic(cubic));
        }
    }

    fn try_arc_with_adapter<I: IntNumber>(
        p0: P,
        arc: &EllipticArc<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Option<Self> {
        let q0 = adapter.float_to_int(&p0);
        let p1 = arc.end_point();
        let q1 = adapter.float_to_int(&p1);
        let center = adapter.float_to_int(&arc.center);
        if q0 == q1 || center == q0 || center == q1 {
            return None;
        }

        Some(NormalizedSegment::Arc(ArcSegment {
            p0,
            p1,
            center: arc.center,
            radii: arc.radii,
            rotation: arc.rotation,
            start_angle: arc.start_angle,
            sweep_angle: arc.sweep_angle,
        }))
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

trait ArcEndPoint<P: FloatPointCompatible> {
    fn end_point(&self) -> P;
}

impl<P: FloatPointCompatible> ArcEndPoint<P> for EllipticArc<P> {
    fn end_point(&self) -> P {
        let angle = self.start_angle + self.sweep_angle;
        let x = self.radii.x() * angle.cos();
        let y = self.radii.y() * angle.sin();
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        P::from_xy(
            self.center.x() + x * cos - y * sin,
            self.center.y() + x * sin + y * cos,
        )
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

trait ShapeFloatRect<P: FloatPointCompatible> {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>>;
}

impl<P: FloatPointCompatible> ShapeFloatRect<P> for CurveShape<P> {
    fn float_rect(&self) -> Option<FloatRect<P::Scalar>> {
        let mut rect = None;
        for contour in &self.contours {
            add_to_rect(&mut rect, contour.start);
            for segment in &contour.segments {
                match *segment {
                    CurveSegment::Line { to } => add_to_rect(&mut rect, to),
                    CurveSegment::Quad { ctrl, to } => {
                        add_to_rect(&mut rect, ctrl);
                        add_to_rect(&mut rect, to);
                    }
                    CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                        add_to_rect(&mut rect, ctrl0);
                        add_to_rect(&mut rect, ctrl1);
                        add_to_rect(&mut rect, to);
                    }
                    CurveSegment::Arc { ref arc } => {
                        add_to_rect(&mut rect, arc.center);
                        add_to_rect(&mut rect, arc.end_point());
                    }
                }
            }
        }
        rect
    }
}

fn add_to_rect<P: FloatPointCompatible>(rect: &mut Option<FloatRect<P::Scalar>>, point: P) {
    match rect {
        Some(rect) => rect.unsafe_add_point(&point),
        None => *rect = Some(FloatRect::with_point(point)),
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
            .build()?;
        let points = [[0.0, 0.0], [1.0, 0.0]];
        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter());

        let segments = shape.to_normalize_segments_with_adapter(ShapeType::Subject, &adapter);

        assert_eq!(segments.len(), 1);
        match &segments[0].normalized_segment {
            NormalizedSegment::Line(segment) => {
                assert_eq!(segment.control_points, [[0.0, 0.0], [1.0, 0.0]])
            }
            _ => panic!("Expected line segment"),
        }

        Ok(())
    }

    #[test]
    fn convert_self_intersecting_cubic_segments() -> Result<(), CurveError> {
        let shape = CurveShapeBuilder::new()
            .move_to([0.0, 0.0])?
            .cubic_to([-3.0, -3.0], [-3.0, -2.0], [-2.0, -2.0])?
            .build()?;

        let segments = shape.to_normalize_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 4);
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
        );

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
