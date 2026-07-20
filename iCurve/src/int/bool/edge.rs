use crate::int::bool::slice::CurveId;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CurveEdge<I: IntNumber> {
    pub(crate) curve: Segment<I>,
    pub(crate) curve_id: CurveId,
}
