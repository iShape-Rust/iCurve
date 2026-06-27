use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, Default)]
pub struct LineSegment<I: IntNumber> {
    pub control_points: [IntPoint<I>; 2],
}
