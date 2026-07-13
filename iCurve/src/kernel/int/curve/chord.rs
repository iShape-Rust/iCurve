use crate::kernel::int::curve::param::SegmentParam;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::vector::IntVector;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SegmentChord<I: IntNumber> {
    pub(crate) a: IntPoint<I>,
    pub(crate) b: IntPoint<I>,
}

impl<I: IntNumber> SegmentChord<I> {
    #[inline]
    pub(crate) fn vector(&self) -> IntVector<I> {
        self.b - self.a
    }

    #[inline]
    pub(crate) fn sqr_length(&self) -> I::Wide {
        self.vector().sqr_length()
    }

    pub(crate) fn param_for_point(&self, point: IntPoint<I>) -> SegmentParam<I> {
        let vector = self.vector();
        let sqr_length = vector.sqr_length();
        debug_assert!(sqr_length > I::Wide::ZERO);

        let projection = (point - self.a).dot_product(vector);
        if projection <= I::Wide::ZERO {
            return SegmentParam::new(I::ZERO);
        }
        if projection >= sqr_length {
            return SegmentParam::new(I::from_wide(SegmentParam::<I>::DENOMINATOR));
        }

        let product = <I::WideUInt as UIntNumber>::Product::multiply(
            projection.unsigned_abs(),
            SegmentParam::<I>::DENOMINATOR.unsigned_abs(),
        );
        let value = product.divide_with_rounding(sqr_length.unsigned_abs());

        SegmentParam::new(I::from_wide(I::Wide::from_uint(value)))
    }
}
