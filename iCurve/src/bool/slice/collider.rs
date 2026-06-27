use crate::kernel::float::curve::segment::FloatSegment;
use crate::kernel::float::math::rect::{ToIntRect, ToRect};
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::rect::IntRect;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Collider<T: FloatNumber, I: IntNumber> {
    pub(super) index: usize,
    pub(super) segment: FloatSegment<T>,
    pub(super) rect: IntRect<I>,
}

impl<T: FloatNumber, I: IntNumber> Collider<T, I> {
    #[inline]
    pub(super) fn new(
        index: usize,
        segment: FloatSegment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Self {
        let rect = segment.to_rect().to_int_rect(adapter);
        Self { index, segment, rect }
    }
}
