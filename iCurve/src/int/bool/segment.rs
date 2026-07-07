use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::int::number::int::IntNumber;
use crate::int::curve::segment::CurveSegment;

pub struct ShapeSegment<I: IntNumber> {
    pub segment: CurveSegment<I>,
    pub shape_type: ShapeType,
}