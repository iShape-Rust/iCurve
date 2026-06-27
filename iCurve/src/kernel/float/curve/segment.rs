use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::line::FloatLineSegment;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub enum FloatSegment<T: FloatNumber> {
    Line(FloatLineSegment<T>),
    Quad(FloatQuadSegment<T>),
    Cubic(FloatCubicSegment<T>),
}

impl<T: FloatNumber> Default for FloatSegment<T> {
    fn default() -> Self {
        Self::Line(FloatLineSegment {
            control_points: [FloatPoint::zero(); 2],
        })
    }
}
