use crate::int::curve::path::CurvePath;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurveShape<I: IntNumber> {
    pub contours: Vec<CurvePath<I>>,
}
