use core::cmp::Ordering;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::number::FloatNumber;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::quad::QuadSegment;

pub enum NormalizedSegment<T: FloatNumber> {
    Line(LineSegment<T>),
    Quad(QuadSegment<T>),
    Cubic(CubicSegment<T>),
}

pub struct Segment<T: FloatNumber> {
    pub normalized_segment: NormalizedSegment<T>,
    pub shape_type: ShapeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SegmentRange<T: FloatNumber> {
    pub(crate) segment_index: usize,
    pub(crate) t0: SegmentParam<T>,
    pub(crate) t1: SegmentParam<T>,
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