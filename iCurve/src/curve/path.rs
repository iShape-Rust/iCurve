use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use crate::curve::segment::CurveSegment;

pub struct CurvePath<P: FloatPointCompatible> {
    pub start: P,
    pub segments: Vec<CurveSegment<P>>,
}