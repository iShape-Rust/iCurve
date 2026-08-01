use crate::collections::stack_vec::StackVec;
use crate::int::CurveInt;
use crate::kernel::int::curve::param::SegmentParam;
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

pub(super) fn solve_unit_quadratic<I: CurveInt>(
    a: I::Wide,
    b: I::Wide,
    c: I::Wide,
) -> StackVec<SegmentParam<I>, 2> {
    let mut roots = StackVec::new();

    if a == I::Wide::ZERO {
        if b != I::Wide::ZERO {
            let t = FixedScale::<I>::div_to_scaled_round(-c, b);
            push_unit_root(&mut roots, t);
        }
        return roots;
    }

    let d = b * b - I::Wide::FOUR * a * c;
    if d < I::Wide::ZERO {
        return roots;
    }

    let denominator = I::Wide::TWO * a;
    let b_scaled = (-b).to_scaled();

    if d == I::Wide::ZERO {
        let t = FixedScale::<I>::div_round(b_scaled, denominator);
        push_unit_root(&mut roots, t);
        return roots;
    }

    // Standard roots:
    //
    //   t0 = (-b - sqrt(D)) / (2a)
    //   t1 = (-b + sqrt(D)) / (2a)
    //
    // The numerator is fixed-scale here. Using integer sqrt(D) would lose
    // the fractional part of sqrt(D) before division; instead compute
    // sqrt(D) * UnitRatio::DENOMINATOR with integer scaled arithmetic.
    let sqrt_d_scaled = scaled_sqrt::<I>(d);

    let t0 = FixedScale::<I>::div_round(b_scaled - sqrt_d_scaled, denominator);
    let t1 = FixedScale::<I>::div_round(b_scaled + sqrt_d_scaled, denominator);

    push_unit_root(&mut roots, t0);
    push_unit_root(&mut roots, t1);

    roots
}

fn scaled_sqrt<I: CurveInt>(value: I::Wide) -> I::Wide {
    debug_assert!(value > I::Wide::ZERO);

    let target_shift = FixedScale::<I>::SHIFT;
    let max_positive_bit = I::WideUInt::LAST_BIT_INDEX - 1;
    let max_shift = max_positive_bit - value.ilog2();
    let even_shift = (target_shift * 2).min(max_shift) & !1;

    // sqrt(value << even_shift) == sqrt(value) << (even_shift / 2).
    // Shift the remaining fixed-scale bits after the square root so the
    // result represents sqrt(value) * UnitRatio::DENOMINATOR.
    let sqrt = (value << even_shift).isqrt();
    let remaining_shift = target_shift - (even_shift >> 1);

    sqrt << remaining_shift
}

fn push_unit_root<I: CurveInt>(roots: &mut StackVec<SegmentParam<I>, 2>, t: I::Wide) {
    // 0 < t < 1
    if t > I::Wide::ZERO && t < SegmentParam::<I>::DENOMINATOR {
        roots.push(SegmentParam::new(I::from_wide(t)));
    }
}
