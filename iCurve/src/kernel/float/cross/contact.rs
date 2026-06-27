use crate::kernel::float::curve::param::FloatSegmentParam;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ContactPoint<T: FloatNumber> {
    pub point: FloatPoint<T>,
    pub t0: FloatSegmentParam<T>,
    pub t1: FloatSegmentParam<T>,
}
