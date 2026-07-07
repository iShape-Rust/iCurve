use i_overlay::i_float::int::number::int::IntNumber;
use crate::kernel::int::curve::param::SegmentParam;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct Range<I: IntNumber> {
    pub(crate) t0: SegmentParam<I>,
    pub(crate) t1: SegmentParam<I>,
}