use crate::curve::contour::CurveContour;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

pub struct CurveShape<P: FloatPointCompatible> {
    pub contours: Vec<CurveContour<P>>,
}
