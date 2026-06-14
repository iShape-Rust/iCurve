use crate::bool::overlay::CurveOverlay;
use crate::bool::segment::SegmentRange;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(crate) fn make_ranges(&self) -> Vec<SegmentRange<P::Scalar>> {
        let ranges = self.simplify_segments();
        // TODO more split logic
        ranges
    }
}
