use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::quad::QuadSegment;

#[derive(Debug, Clone, Copy)]
pub enum Segment<I: IntNumber> {
    Line(LineSegment<I>),
    Quad(QuadSegment<I>),
    Cubic(CubicSegment<I>),
}

impl<I: IntNumber> Default for Segment<I> {
    fn default() -> Self {
        Self::Line(LineSegment {
            control_points: [IntPoint::ZERO; 2],
        })
    }
}