use crate::kernel::int::curve::segment::Segment;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::int::number::int::IntNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CurveId(pub(crate) usize);

#[derive(Debug, Clone, Copy)]
pub(crate) struct CurveSlice<I: IntNumber> {
    pub(crate) curve: Segment<I>,
    pub(crate) shape_type: ShapeType,
}
