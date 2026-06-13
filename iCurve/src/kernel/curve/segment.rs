use i_overlay::i_float::float::number::FloatNumber;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;

pub enum Segment<T: FloatNumber> {
    Line(LineSegment<T>),
    Quad(QuadSegment<T>),
    Cubic(CubicSegment<T>),
}
