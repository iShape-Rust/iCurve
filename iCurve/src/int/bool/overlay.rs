use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use crate::int::bool::segment::ShapeSegment;

pub struct IntCurveOverlay<I: IntNumber> {
    pub(crate) segments: Vec<ShapeSegment<I>>,
    
    
}