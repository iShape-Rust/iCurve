use crate::int::CurveInt;
use i_overlay::i_float::int::number::unit_ratio::UnitRatio;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

/// Fixed-point parameter in the inclusive range from the start to the end of a segment.
pub type SegmentParam<I> = UnitRatio<I>;

#[inline]
pub(crate) fn interpolate_segment_param<I: CurveInt>(
    start: SegmentParam<I>,
    end: SegmentParam<I>,
    local: SegmentParam<I>,
) -> SegmentParam<I> {
    let denominator = SegmentParam::<I>::DENOMINATOR;
    let span = end.value() - start.value();
    debug_assert!(span >= I::Wide::ZERO);

    let offset = (span * local.value() + (denominator >> 1)) / denominator;
    SegmentParam::new(I::from_wide(start.value() + offset))
}
