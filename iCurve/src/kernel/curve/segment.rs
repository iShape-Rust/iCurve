use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;

#[derive(Debug, Clone, Copy)]
pub enum Segment<T: FloatNumber> {
    Line(LineSegment<T>),
    Quad(QuadSegment<T>),
    Cubic(CubicSegment<T>),
}
