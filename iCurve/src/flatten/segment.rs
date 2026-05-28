use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

pub enum NormalizedSegment<P: FloatPointCompatible> {
    Line(LineSegment<P>),
    Quad(QuadSegment<P>),
    Cubic(CubicSegment<P>),
    Arc(ArcSegment<P>),
}

pub struct Segment<P: FloatPointCompatible> {
    pub normalized_segment: NormalizedSegment<P>,
    pub shape_type: ShapeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SubSegment<T: FloatNumber> {
    pub(crate) segment_index: usize,
    pub(crate) t0: T,
    pub(crate) t1: T,
}
pub struct LineSegment<P: FloatPointCompatible> {
    pub control_points: [P; 2],
}

pub struct QuadSegment<P: FloatPointCompatible> {
    pub control_points: [P; 3],
}

pub struct CubicSegment<P: FloatPointCompatible> {
    pub control_points: [P; 4],
}

pub struct ArcSegment<P: FloatPointCompatible> {
    pub p0: P,
    pub p1: P,
    pub center: P,
    pub radii: P,
    pub rotation: P::Scalar,
    pub start_angle: P::Scalar,
    pub sweep_angle: P::Scalar,
}
