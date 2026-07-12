use crate::kernel::int::curve::bisect::Bisect;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::vector::IntVector;
use i_overlay::i_shape::int::IntPoint;

pub(super) struct Split<I: IntNumber> {
    pub(super) t0: SegmentParam<I>,
    pub(super) s0: Segment<I>,
    pub(super) t1: SegmentParam<I>,
    pub(super) s1: Segment<I>,
}

impl<I: IntNumber> Segment<I> {
    #[inline]
    pub(super) fn base_vector(&self) -> IntVector<I> {
        let [a, b] = self.ends();
        a - b
    }

    #[inline]
    pub(super) fn split(&self, t: SegmentParam<I>, a: IntPoint<I>, b: IntPoint<I>) -> Split<I> {
        let half = t.value() >> 1;
        let t0 = SegmentParam::new(I::from_wide(t.value() - half));
        let t1 = SegmentParam::new(I::from_wide(t.value() + half));

        let [s0, s1] = self.bisect(a, b, t);

        Split { t0, s0, t1, s1 }
    }
}
