use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub enum CurveSegment<I: IntNumber> {
    Line {
        to: IntPoint<I>,
    },
    Quad {
        ctrl: IntPoint<I>,
        to: IntPoint<I>,
    },
    Cubic {
        ctrl0: IntPoint<I>,
        ctrl1: IntPoint<I>,
        to: IntPoint<I>,
    },
}

impl<I: IntNumber> CurveSegment<I> {}
