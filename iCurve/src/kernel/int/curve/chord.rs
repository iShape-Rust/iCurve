use crate::kernel::int::curve::param::SegmentParam;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::product_uint::UIntProduct;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
#[cfg(test)]
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_float::int::vector::IntVector;
use i_overlay::i_shape::int::IntPoint;

pub(crate) trait Chord<I: IntNumber> {
    fn chord(&self) -> SegmentChord<I>;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SegmentChord<I: IntNumber> {
    pub(crate) a: IntPoint<I>,
    pub(crate) b: IntPoint<I>,
}

impl<I: IntNumber> SegmentChord<I> {
    #[inline]
    pub(crate) fn is_zero_length(&self) -> bool {
        self.b == self.a
    }

    #[inline]
    pub(crate) fn vector(&self) -> IntVector<I> {
        self.b - self.a
    }

    #[inline]
    pub(crate) fn sqr_length(&self) -> I::WideUInt {
        self.vector().sqr_length()
    }

    #[inline]
    #[cfg(test)]
    pub(crate) fn to_rect(self) -> IntRect<I> {
        IntRect::with_ab(self.a, self.b)
    }

    pub(crate) fn param_for_point(&self, point: IntPoint<I>) -> SegmentParam<I> {
        let vector = self.vector();
        let sqr_length = vector.sqr_length();
        debug_assert!(sqr_length > I::WideUInt::ZERO);

        let projection = (point - self.a).dot_product(vector);
        if projection <= I::Wide::ZERO {
            return SegmentParam::new(I::ZERO);
        }
        let projection = projection.to_uint();
        if projection >= sqr_length {
            return SegmentParam::new(I::from_wide(SegmentParam::<I>::DENOMINATOR));
        }

        let product = <I::WideUInt as UIntNumber>::Product::multiply(
            projection,
            SegmentParam::<I>::DENOMINATOR.to_uint(),
        );
        let value = product.divide_with_rounding(sqr_length);

        SegmentParam::new(I::from_wide(I::Wide::from_uint(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_projection<I: IntNumber>() {
        let point = |x: I| IntPoint::new(x, I::ZERO);
        let chord = SegmentChord {
            a: point(I::ZERO),
            b: point(I::from_u32(3)),
        };
        let denominator = SegmentParam::<I>::DENOMINATOR;

        assert!(chord.param_for_point(point(-I::ONE)).value() == I::Wide::ZERO);
        assert!(chord.param_for_point(point(I::ZERO)).value() == I::Wide::ZERO);
        assert!(chord.param_for_point(chord.b).value() == denominator);
        assert!(chord.param_for_point(point(I::FOUR)).value() == denominator);
        assert!(chord.param_for_point(point(I::ONE)).value() == denominator / I::Wide::from_u32(3));
        assert!(
            chord.param_for_point(point(I::TWO)).value()
                == (I::Wide::TWO * denominator + I::Wide::ONE) / I::Wide::from_u32(3)
        );

        let reversed = SegmentChord {
            a: chord.b,
            b: chord.a,
        };
        assert!(reversed.param_for_point(point(I::FOUR)).value() == I::Wide::ZERO);
        assert!(reversed.param_for_point(point(-I::ONE)).value() == denominator);
    }

    #[test]
    fn projects_points_with_signed_clamping_for_all_engines() {
        check_projection::<i16>();
        check_projection::<i32>();
        check_projection::<i64>();
    }

    fn check_coordinate_limit<I: IntNumber>() {
        let limit = I::ONE << (I::BITS - crate::int::CURVE_COORDINATE_SAFETY_BITS);
        let chord = SegmentChord {
            a: IntPoint::new(-limit, -limit),
            b: IntPoint::new(limit, limit),
        };
        let expected_length = (limit.to_wide() * limit.to_wide()).to_uint() << 3;

        assert!(chord.sqr_length() == expected_length);
        assert!(chord.sqr_length() < I::WideUInt::LAST_BIT);
        assert!(chord.param_for_point(IntPoint::ZERO).value() == SegmentParam::<I>::half().value());
    }

    #[test]
    fn projects_near_coordinate_limit_for_all_engines() {
        check_coordinate_limit::<i16>();
        check_coordinate_limit::<i32>();
        check_coordinate_limit::<i64>();
    }
}
