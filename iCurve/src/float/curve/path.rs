use crate::float::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

pub struct CurvePath<P: FloatPointCompatible> {
    pub start: P,
    pub segments: Vec<CurveSegment<P>>,
}
