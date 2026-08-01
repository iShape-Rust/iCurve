use crate::int::CurveInt;
use crate::int::bool::source::CurveId;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::segment::Segment;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CurveEdge<I: CurveInt> {
    pub(crate) curve: Segment<I>,
    pub(crate) curve_id: CurveId,
    // Source parameters cannot be recovered from endpoints because distinct
    // fixed-point positions may quantize to the same integer point.
    pub(crate) start_param: SegmentParam<I>,
    pub(crate) end_param: SegmentParam<I>,
}

impl<I: CurveInt> CurveEdge<I> {
    #[inline]
    pub(crate) fn new(
        curve: Segment<I>,
        curve_id: CurveId,
        start_param: SegmentParam<I>,
        end_param: SegmentParam<I>,
    ) -> Self {
        Self {
            curve,
            curve_id,
            start_param,
            end_param,
        }
    }

    #[cfg(test)]
    pub(crate) fn full(curve: Segment<I>, curve_id: CurveId) -> Self {
        Self::new(
            curve,
            curve_id,
            SegmentParam::new(I::ZERO),
            SegmentParam::new(I::from_wide(SegmentParam::<I>::DENOMINATOR)),
        )
    }
}
