use crate::int::bool::segment::ShapeSegment;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;

pub struct IntCurveOverlay<I: IntNumber> {
    pub(crate) segments: Vec<ShapeSegment<I>>,
}
