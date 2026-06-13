use crate::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use crate::curve::path::CurvePath;

pub struct CurveContour<P: FloatPointCompatible> {
    pub start: P,
    pub segments: Vec<CurveSegment<P>>,
}

impl<P: FloatPointCompatible> From<CurvePath<P>> for CurveContour<P> {
    fn from(value: CurvePath<P>) -> Self {
        CurveContour {
            start: value.start,
            segments: value.segments,
        }
    }
}