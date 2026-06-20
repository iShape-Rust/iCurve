use crate::kernel::curve::param::SegmentParam;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub struct QuadSegment<T: FloatNumber> {
    pub control_points: [FloatPoint<T>; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct SubQuadSegment<T: FloatNumber> {
    pub quad: QuadSegment<T>,
    pub t0: SegmentParam<T>,
    pub t1: SegmentParam<T>,
}

impl<T: FloatNumber> SubQuadSegment<T> {
    #[inline]
    pub fn with_quad(quad: QuadSegment<T>) -> Self {
        Self {
            quad,
            t0: SegmentParam::Start,
            t1: SegmentParam::End,
        }
    }
}
