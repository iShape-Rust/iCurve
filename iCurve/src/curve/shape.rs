use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use crate::curve::path::CurvePath;

pub struct CurveShape<P: FloatPointCompatible> {
    pub contours: Vec<CurvePath<P>>,
}
