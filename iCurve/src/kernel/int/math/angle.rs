use crate::int::CurveInt;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::vector::IntVector;

pub(crate) trait ApproximateAngle {
    fn is_nearly_collinear_with(&self, other: Self, sin_angle_neg_pow2: u32) -> bool;
}

impl<I: CurveInt> ApproximateAngle for IntVector<I> {
    fn is_nearly_collinear_with(&self, other: Self, sin_angle_neg_pow2: u32) -> bool {
        let abs_cross = self.cross_product(other).unsigned_abs();
        if abs_cross == I::WideUInt::ZERO {
            return true;
        }
        let log_cross = abs_cross.ilog2();
        let sqr_len_0 = self.sqr_length();
        let sqr_len_1 = other.sqr_length();
        debug_assert!(sqr_len_0 != I::Wide::ZERO);
        debug_assert!(sqr_len_1 != I::Wide::ZERO);

        let log_len = (sqr_len_0.ilog2() + sqr_len_1.ilog2()) >> 1;

        if log_len < log_cross {
            return false;
        }

        log_len - log_cross >= sin_angle_neg_pow2
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::int::math::angle::ApproximateAngle;
    use i_overlay::i_float::int::vector::IntVector;

    #[test]
    fn test_0() {
        let a: IntVector<i32> = IntVector::new(8, 0);
        let b: IntVector<i32> = IntVector::new(8, 1);
        let c: IntVector<i32> = IntVector::new(8, 2);
        let d: IntVector<i32> = IntVector::new(8, 3);
        let e: IntVector<i32> = IntVector::new(8, 4);
        let f: IntVector<i32> = IntVector::new(8, 5);

        assert!(a.is_nearly_collinear_with(b, 3));
        assert!(!a.is_nearly_collinear_with(c, 3));
        assert!(!a.is_nearly_collinear_with(d, 3));
        assert!(!a.is_nearly_collinear_with(e, 3));
        assert!(!a.is_nearly_collinear_with(f, 3));
    }
}
