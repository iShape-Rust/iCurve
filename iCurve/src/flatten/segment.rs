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
pub(crate) struct SegmentRange<T: FloatNumber> {
    pub(crate) segment_index: usize,
    pub(crate) t0: SegmentParam<T>,
    pub(crate) t1: SegmentParam<T>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SegmentParam<T: FloatNumber> {
    Start,
    Inner(T),
    End,
}

impl<T: FloatNumber> SegmentRange<T> {
    #[inline(always)]
    pub(crate) fn new(segment_index: usize, t0: T, t1: T) -> Self {
        Self {
            segment_index,
            t0: SegmentParam::new(t0),
            t1: SegmentParam::new(t1),
        }
    }

    #[inline(always)]
    pub(crate) fn full(segment_index: usize) -> Self {
        Self {
            segment_index,
            t0: SegmentParam::Start,
            t1: SegmentParam::End,
        }
    }
}

impl<T: FloatNumber> SegmentParam<T> {
    #[inline(always)]
    pub(crate) fn new(t: T) -> Self {
        if t == T::from_float(0.0) {
            Self::Start
        } else if t == T::from_float(1.0) {
            Self::End
        } else {
            Self::Inner(t)
        }
    }

    #[inline(always)]
    pub(crate) fn value(self) -> T {
        match self {
            Self::Start => T::from_float(0.0),
            Self::Inner(t) => t,
            Self::End => T::from_float(1.0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct LineSegment<P: FloatPointCompatible> {
    pub control_points: [P; 2],
}

#[derive(Clone, Copy)]
pub struct QuadSegment<P: FloatPointCompatible> {
    pub control_points: [P; 3],
}

#[derive(Clone, Copy)]
pub struct CubicSegment<P: FloatPointCompatible> {
    pub control_points: [P; 4],
}

#[derive(Clone, Copy)]
pub struct ArcSegment<P: FloatPointCompatible> {
    pub p0: P,
    pub p1: P,
    pub center: P,
    pub radii: P,
    pub rotation: P::Scalar,
    pub start_angle: P::Scalar,
    pub sweep_angle: P::Scalar,
}
