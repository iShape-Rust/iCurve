use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

pub(crate) struct Segment<P: FloatPointCompatible> {
    pub(crate) segment_kind: SegmentKind<P>,
    pub(crate) shape_type: ShapeType,
}

pub(crate) enum SegmentKind<P: FloatPointCompatible> {
    Line(LineSegment<P>),
    Quad(QuadSegment<P>),
    Cubic(CubicSegment<P>),
    Arc(ArcSegment<P>),
}

pub(crate) struct SubSegment<T: FloatNumber> {
    pub(crate) segment_index: usize,
    pub(crate) t0: T,
    pub(crate) t1: T,
}
pub(crate) struct LineSegment<P: FloatPointCompatible> {
    pub(crate) control_points: [P; 2],
}

pub(crate) struct QuadSegment<P: FloatPointCompatible> {
    pub(crate) control_points: [P; 3],
}

pub(crate) struct CubicSegment<P: FloatPointCompatible> {
    pub(crate) control_points: [P; 4],
}

pub(crate) struct ArcSegment<P: FloatPointCompatible> {
    pub(crate) p0: P,
    pub(crate) p1: P,
    pub(crate) center: P,
    pub(crate) radii: P,
    pub(crate) rotation: P::Scalar,
    pub(crate) start_angle: P::Scalar,
    pub(crate) sweep_angle: P::Scalar,
}
