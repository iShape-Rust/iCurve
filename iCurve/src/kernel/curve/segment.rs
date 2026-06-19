use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub enum Segment<T: FloatNumber> {
    Line(LineSegment<T>),
    Quad(QuadSegment<T>),
    Cubic(CubicSegment<T>),
}

impl<T: FloatNumber> Default for Segment<T> {
    fn default() -> Self {
        Self::Line(LineSegment {
            control_points: [FloatPoint::zero(); 2],
        })
    }
}
