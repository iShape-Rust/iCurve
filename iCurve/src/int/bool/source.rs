use crate::kernel::int::curve::segment::Segment;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::int::number::int::IntNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct CurveId(pub(crate) usize);

#[derive(Debug, Clone)]
pub(crate) struct CurveSource<I: IntNumber> {
    pub(crate) curve: Segment<I>,
    pub(crate) shape_type: ShapeType,
}

impl<I: IntNumber> CurveSource<I> {
    pub(crate) fn new(curve: Segment<I>, shape_type: ShapeType) -> Self {
        Self { curve, shape_type }
    }
}
