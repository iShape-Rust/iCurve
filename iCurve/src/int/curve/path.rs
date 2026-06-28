use crate::int::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub struct CurvePath<I: IntNumber> {
    pub start: IntPoint<I>,
    pub segments: Vec<CurveSegment<I>>,
}
