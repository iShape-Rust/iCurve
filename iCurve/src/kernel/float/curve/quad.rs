use crate::kernel::float::curve::param::FloatSegmentParam;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub struct FloatQuadSegment<T: FloatNumber> {
    pub control_points: [FloatPoint<T>; 3],
}

impl<T: FloatNumber> Default for FloatQuadSegment<T> {
    #[inline]
    fn default() -> Self {
        Self {
            control_points: [FloatPoint::zero(); 3],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SubQuadSegment<T: FloatNumber> {
    pub quad: FloatQuadSegment<T>,
    pub t0: FloatSegmentParam<T>,
    pub t1: FloatSegmentParam<T>,
}

impl<T: FloatNumber> SubQuadSegment<T> {
    #[inline]
    pub fn with_quad(quad: FloatQuadSegment<T>) -> Self {
        Self {
            quad,
            t0: FloatSegmentParam::Start,
            t1: FloatSegmentParam::End,
        }
    }
}
