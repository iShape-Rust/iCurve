use alloc::vec::Vec;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

use crate::curve::arc::EllipticArc;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::segment::{ArcSegment, CubicSegment, LineSegment, QuadSegment, Segment, SegmentKind};

pub(crate) trait ShapeToSegments<P: FloatPointCompatible> {
    fn to_segments(&self, shape_type: ShapeType) -> Vec<Segment<P>>;
}

impl<P: FloatPointCompatible> ShapeToSegments<P> for CurveShape<P> {
    fn to_segments(&self, shape_type: ShapeType) -> Vec<Segment<P>> {
        let mut result = Vec::with_capacity(self.segments_count());

        for contour in &self.contours {
            let mut current = contour.start;

            for curve_segment in &contour.segments {
                let (segment_kind, next) = curve_segment.to_segment_kind(current);

                result.push(Segment {
                    segment_kind,
                    shape_type,
                });

                current = next;
            }
        }

        result
    }
}

trait CurveSegmentToKind<P: FloatPointCompatible> {
    fn to_segment_kind(&self, p0: P) -> (SegmentKind<P>, P);
}

impl<P: FloatPointCompatible> CurveSegmentToKind<P> for CurveSegment<P> {
    fn to_segment_kind(&self, p0: P) -> (SegmentKind<P>, P) {
        match *self {
            CurveSegment::Line { to } => (
                SegmentKind::Line(LineSegment {
                    control_points: [p0, to],
                }),
                to,
            ),
            CurveSegment::Quad { ctrl, to } => (
                SegmentKind::Quad(QuadSegment {
                    control_points: [p0, ctrl, to],
                }),
                to,
            ),
            CurveSegment::Cubic { ctrl0, ctrl1, to } => (
                SegmentKind::Cubic(CubicSegment {
                    control_points: [p0, ctrl0, ctrl1, to],
                }),
                to,
            ),
            CurveSegment::Arc { ref arc } => {
                let p1 = arc.end_point();
                (
                    SegmentKind::Arc(ArcSegment {
                        p0,
                        p1,
                        center: arc.center,
                        radii: arc.radii,
                        rotation: arc.rotation,
                        start_angle: arc.start_angle,
                        sweep_angle: arc.sweep_angle,
                    }),
                    p1,
                )
            }
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

        let segments = shape.to_segments(ShapeType::Subject);

        assert_eq!(segments.len(), 3);

        match &segments[0].segment_kind {
            SegmentKind::Line(segment) => assert_eq!(segment.control_points, [[0.0, 0.0], [1.0, 0.0]]),
            _ => panic!("Expected line segment"),
        }

        match &segments[1].segment_kind {
            SegmentKind::Quad(segment) => {
                assert_eq!(segment.control_points, [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
            }
            _ => panic!("Expected quad segment"),
        }

        match &segments[2].segment_kind {
            SegmentKind::Cubic(segment) => assert_eq!(
                segment.control_points,
                [[0.0, 1.0], [-1.0, 1.0], [-1.0, 0.0], [0.0, 0.0]]
            ),
            _ => panic!("Expected cubic segment"),
        }

        Ok(())
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

        let segments = shape.to_segments(ShapeType::Subject);

        match &segments[0].segment_kind {
            SegmentKind::Arc(segment) => {
                assert_eq!(segment.p0, [1.0, 0.0]);
                assert!(segment.p1[0].abs() < 0.000001);
                assert!((segment.p1[1] - 1.0).abs() < 0.000001);
            }
            _ => panic!("Expected arc segment"),
        }

        Ok(())
    }
}
