use crate::kernel::float::curve::param::FloatSegmentParam;
use crate::kernel::float::curve::segment::FloatSegment;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::number::FloatNumber;

pub struct ShapeSegment<T: FloatNumber> {
    pub segment: FloatSegment<T>,
    pub shape_type: ShapeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SegmentRange<T: FloatNumber> {
    pub(crate) segment_index: usize,
    pub(crate) t0: FloatSegmentParam<T>,
    pub(crate) t1: FloatSegmentParam<T>,
}

impl<T: FloatNumber> SegmentRange<T> {
    #[inline(always)]
    pub(crate) fn new(segment_index: usize, t0: T, t1: T) -> Self {
        Self {
            segment_index,
            t0: FloatSegmentParam::new(t0),
            t1: FloatSegmentParam::new(t1),
        }
    }

    #[inline(always)]
    pub(crate) fn full(segment_index: usize) -> Self {
        Self {
            segment_index,
            t0: FloatSegmentParam::Start,
            t1: FloatSegmentParam::End,
        }
    }
}
