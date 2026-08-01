use crate::int::CurveInt;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::core::overlay::ShapeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct CurveId(pub(crate) usize);

#[derive(Debug, Clone)]
pub(crate) struct CurveSource<I: CurveInt> {
    pub(crate) curve: Segment<I>,
    pub(crate) shape_type: ShapeType,
}

impl<I: CurveInt> CurveSource<I> {
    pub(crate) fn new(curve: Segment<I>, shape_type: ShapeType) -> Self {
        Self { curve, shape_type }
    }
}
