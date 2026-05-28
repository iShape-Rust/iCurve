use crate::bool::overlay::CurveOverlay;
use crate::curve::arc::EllipticArc;
use crate::curve::contour::CurveContour;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, SubSegment,
};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::vector::edge::{DataVectorPath, DataVectorShape};

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(super) fn recombine(
        &self,
        vector_shapes: Vec<DataVectorShape<I, SubSegment<P::Scalar>>>,
    ) -> Vec<CurveShape<P>> {
        let mut shapes = Vec::with_capacity(vector_shapes.len());

        for vector_shape in vector_shapes {
            let mut contours = Vec::with_capacity(vector_shape.len());

            for vector_path in vector_shape {
                if let Some(contour) = self.recombine_path(vector_path) {
                    contours.push(contour);
                }
            }

            if !contours.is_empty() {
                shapes.push(CurveShape { contours });
            }
        }

        shapes
    }

    fn recombine_path(
        &self,
        vector_path: DataVectorPath<I, SubSegment<P::Scalar>>,
    ) -> Option<CurveContour<P>> {
        let mut start = None;
        let mut segments = Vec::with_capacity(vector_path.len());

        for edge in vector_path {
            let normalized = &self.segments[edge.data.segment_index].normalized_segment;
            let Some((piece_start, piece)) = normalized.to_curve_piece(edge.data) else {
                continue;
            };

            if start.is_none() {
                start = Some(piece_start);
            }
            segments.push(piece);
        }

        if segments.is_empty() {
            return None;
        }

        Some(CurveContour {
            start: start.expect("non-empty segment list must set contour start"),
            segments,
        })
    }
}

trait CurvePiece<P: FloatPointCompatible> {
    fn to_curve_piece(&self, range: SubSegment<P::Scalar>) -> Option<(P, CurveSegment<P>)>;
}

impl<P: FloatPointCompatible> CurvePiece<P> for NormalizedSegment<P> {
    fn to_curve_piece(&self, range: SubSegment<P::Scalar>) -> Option<(P, CurveSegment<P>)> {
        match self {
            Self::Line(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                Some((
                    segment.control_points[0],
                    CurveSegment::Line {
                        to: segment.control_points[1],
                    },
                ))
            }
            Self::Quad(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                Some((
                    segment.control_points[0],
                    CurveSegment::Quad {
                        ctrl: segment.control_points[1],
                        to: segment.control_points[2],
                    },
                ))
            }
            Self::Cubic(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                Some((
                    segment.control_points[0],
                    CurveSegment::Cubic {
                        ctrl0: segment.control_points[1],
                        ctrl1: segment.control_points[2],
                        to: segment.control_points[3],
                    },
                ))
            }
            Self::Arc(segment) => {
                if range.t0 == range.t1 {
                    return None;
                }

                let start = segment.point_at(range.t0);
                let arc = EllipticArc {
                    center: segment.center,
                    radii: segment.radii,
                    rotation: segment.rotation,
                    start_angle: segment.start_angle + segment.sweep_angle * range.t0,
                    sweep_angle: segment.sweep_angle * (range.t1 - range.t0),
                };

                Some((start, CurveSegment::Arc { arc }))
            }
        }
    }
}

trait SegmentRange<P: FloatPointCompatible>: SplitAt<P::Scalar, Output = [Self; 2]> + Sized {
    fn range(&self, t0: P::Scalar, t1: P::Scalar) -> Option<Self> {
        if t0 == t1 {
            return None;
        }

        if t0 < t1 {
            Some(self.forward_range(t0, t1))
        } else {
            Some(self.forward_range(t1, t0).reversed())
        }
    }

    fn forward_range(&self, t0: P::Scalar, t1: P::Scalar) -> Self {
        let zero = P::Scalar::from_float(0.0);
        let one = P::Scalar::from_float(1.0);

        if t0 <= zero {
            let [segment, _] = self.split_at(t1);
            return segment;
        }

        let [_, right] = self.split_at(t0);
        let local_t = (t1 - t0) / (one - t0);
        let [segment, _] = right.split_at(local_t);
        segment
    }

    fn reversed(self) -> Self;
}

impl<P: FloatPointCompatible> SegmentRange<P> for LineSegment<P> {
    fn reversed(self) -> Self {
        let [p0, p1] = self.control_points;
        Self {
            control_points: [p1, p0],
        }
    }
}

impl<P: FloatPointCompatible> SegmentRange<P> for QuadSegment<P> {
    fn reversed(self) -> Self {
        let [p0, p1, p2] = self.control_points;
        Self {
            control_points: [p2, p1, p0],
        }
    }
}

impl<P: FloatPointCompatible> SegmentRange<P> for CubicSegment<P> {
    fn reversed(self) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        Self {
            control_points: [p3, p2, p1, p0],
        }
    }
}

trait ArcPointAt<P: FloatPointCompatible> {
    fn point_at(&self, t: P::Scalar) -> P;
}

impl<P: FloatPointCompatible> ArcPointAt<P> for ArcSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        let angle = self.start_angle + self.sweep_angle * t;
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
