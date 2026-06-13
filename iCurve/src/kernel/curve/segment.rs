use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;

pub enum Segment<T: FloatNumber> {
    Line(LineSegment<T>),
    Quad(QuadSegment<T>),
    Cubic(CubicSegment<T>),
}
