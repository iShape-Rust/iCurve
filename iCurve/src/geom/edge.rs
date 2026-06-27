use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub(crate) struct IntEdge<I: IntNumber> {
    a: IntPoint<I>,
    b: IntPoint<I>,
}

impl<I: IntNumber> IntEdge<I> {}
