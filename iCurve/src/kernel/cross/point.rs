use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use crate::kernel::curve::param::SegmentParam;

pub struct CrossPoint<T: FloatNumber> {
    pub point: FloatPoint<T>,
    pub t0: SegmentParam<T>,
    pub t1: SegmentParam<T>
}