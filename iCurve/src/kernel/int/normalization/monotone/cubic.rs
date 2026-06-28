use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::normalization::monotone::decomposition::{
    DecomposeIntoMonotone, MonotoneDecompositionDirection, roots_to_segments,
};
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

impl<I: IntNumber> DecomposeIntoMonotone for CubicSegment<I> {
    type Output = StackVec<CubicSegment<I>, 5>;

    fn decompose_into_monotone(&self) -> Self::Output {
        let x_roots = self.monotone_roots_by_direction(MonotoneDecompositionDirection::X);
        let y_roots = self.monotone_roots_by_direction(MonotoneDecompositionDirection::Y);

        let mut roots = StackVec::<_, 4>::new();
        roots.extend_from_slice(x_roots.as_slice());
        roots.extend_from_slice(y_roots.as_slice());

        roots_to_segments(self, roots)
    }
}

impl<I: IntNumber> CubicSegment<I> {
    fn monotone_roots_by_direction(
        &self,
        direction: MonotoneDecompositionDirection,
    ) -> StackVec<SegmentParam<I>, 2> {
        let [a, b, c, _] = Self::axis_abcd(self.control_points, direction);

        // Cubic Bezier in Bernstein form:
        //
        //   q(t) = (1 - t)^3 * k0
        //        + 3 * (1 - t)^2 * t * k1
        //        + 3 * (1 - t) * t^2 * k2
        //        + t^3 * k3
        //
        // In power form:
        //
        //   q(t) = a * t^3 + b * t^2 + c * t + d
        //
        // A cubic is monotone on intervals that do not contain an extremum.
        // Extrema are roots of the derivative:
        //
        //   q'(t) = 3 * a * t^2 + 2 * b * t + c = 0
        //
        // The derivative is quadratic, so it can contribute up to two split
        // parameters per axis.
        solve_unit_quadratic::<I>(I::Wide::from_u32(3) * a, I::Wide::TWO * b, c)
    }

    #[inline]
    fn axis_abcd(
        control_points: [IntPoint<I>; 4],
        direction: MonotoneDecompositionDirection,
    ) -> [I::Wide; 4] {
        let [k0, k1, k2, k3] = match direction {
            MonotoneDecompositionDirection::X => control_points.map(|p| p.x.to_wide()),
            MonotoneDecompositionDirection::Y => control_points.map(|p| p.y.to_wide()),
        };

        // Expanding the Bernstein cubic gives:
        //
        //   q(t) = a * t^3 + b * t^2 + c * t + d
        //
        // where:
        //
        //   a = -k0 + 3 * k1 - 3 * k2 + k3
        //   b =  3 * k0 - 6 * k1 + 3 * k2
        //   c = -3 * k0 + 3 * k1
        //   d =  k0
        let three = I::Wide::from_u32(3);
        let six = I::Wide::from_u32(6);

        [
            k3 - three * k2 + three * k1 - k0,
            three * k0 - six * k1 + three * k2,
            three * (k1 - k0),
            k0,
        ]
    }
}

fn solve_unit_quadratic<I: IntNumber>(a: I::Wide, b: I::Wide, c: I::Wide) -> StackVec<SegmentParam<I>, 2> {
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

fn scaled_sqrt<I: IntNumber>(value: I::Wide) -> I::Wide {
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

fn push_unit_root<I: IntNumber>(roots: &mut StackVec<SegmentParam<I>, 2>, t: I::Wide) {
    // 0 < t < 1
    if t > I::Wide::ZERO && t < SegmentParam::<I>::DENOMINATOR {
        roots.push(SegmentParam::new(I::from_wide(t)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_cubic_by_distinct_axis_extrema() {
        let cubic = CubicSegment::<i32> {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(4, 3),
                IntPoint::new(-1, -3),
                IntPoint::new(3, 0),
            ],
        };

        let parts = cubic.decompose_into_monotone();
        let parts = parts.as_slice();

        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].control_points[0], cubic.control_points[0]);
        assert_eq!(parts[4].control_points[3], cubic.control_points[3]);
    }

    #[test]
    fn keeps_fractional_derivative_roots_in_fixed_scale() {
        let cubic = CubicSegment::<i32> {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 4),
                IntPoint::new(0, -1),
                IntPoint::new(0, 3),
            ],
        };

        let roots = cubic.monotone_roots_by_direction(MonotoneDecompositionDirection::Y);
        let roots = roots.as_slice();

        assert_eq!(roots.len(), 2);
        assert!((roots[0].value() - SegmentParam::<i32>::from_int(1, 3).value()).abs() <= 1i64);
        assert!((roots[1].value() - SegmentParam::<i32>::from_int(2, 3).value()).abs() <= 1i64);
    }

    #[test]
    fn keeps_irrational_derivative_roots_in_fixed_scale() {
        let cubic = CubicSegment::<i32> {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 3),
                IntPoint::new(0, -3),
                IntPoint::new(0, 0),
            ],
        };

        let roots = cubic.monotone_roots_by_direction(MonotoneDecompositionDirection::Y);
        let roots = roots.as_slice();

        let scale = SegmentParam::<i32>::DENOMINATOR as f64;
        let offset = (1.0_f64 / 3.0).sqrt();
        let expected_0 = (scale * (1.0 - offset) * 0.5).round() as i64;
        let expected_1 = (scale * (1.0 + offset) * 0.5).round() as i64;

        assert_eq!(roots.len(), 2);
        assert!((roots[0].value() - expected_0).abs() <= 1i64);
        assert!((roots[1].value() - expected_1).abs() <= 1i64);
    }
}
