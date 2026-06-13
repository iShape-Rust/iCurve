use crate::bool::overlay::CurveOverlay;
use crate::flatten::segment::SegmentRange;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(crate) fn overlay_ranges(&self, ranges: &mut Vec<SegmentRange<P::Scalar>>) {}
}
