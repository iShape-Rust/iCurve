use crate::kernel::curve::param::SegmentParam;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ContactPoint<T: FloatNumber> {
    pub point: FloatPoint<T>,
    pub t0: SegmentParam<T>,
    pub t1: SegmentParam<T>,
}
