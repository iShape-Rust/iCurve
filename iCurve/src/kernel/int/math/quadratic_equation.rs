use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

pub(crate) struct QuadraticEquation;

impl QuadraticEquation {
    pub(crate) fn solve<I: IntNumber>(a: I, b: I, c: I) -> Option<[I; 2]> {
        if a == I::ZERO {
            return if b == I::ZERO {
                None
            } else {
                let t = -c / b;
                Some([t, t])
            };
        }

        let d = b * b - I::FOUR * a * c;
        if d < I::ZERO {
            return None;
        }

        let two_a = I::Wide::TWO * a.to_wide();
        let b_scaled = b.to_scaled_wide();

        if d == I::ZERO {
            let t = FixedScale::<I>::to_int_round(-b_scaled / two_a);
            return Some([t, t]);
        }

        let sqrt_dw = d.scaled_isqrt();
        let bw = b.to_scaled_wide();

        let x0 = FixedScale::<I>::to_int_round((-bw - sqrt_dw) / two_a);
        let x1 = FixedScale::<I>::to_int_round((-bw + sqrt_dw) / two_a);

        Some([x0, x1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_two_integer_roots() {
        let roots = QuadraticEquation::solve(1i32, -4, 3).unwrap();

        assert_eq!(roots, [1, 3]);
    }

    #[test]
    fn solves_single_integer_root() {
        let roots = QuadraticEquation::solve(1i32, -2, 1).unwrap();

        assert_eq!(roots, [1, 1]);
    }

    #[test]
    fn solves_linear_equation() {
        let roots = QuadraticEquation::solve(0i32, 2, -4).unwrap();

        assert_eq!(roots, [2, 2]);
    }

    #[test]
    fn ignores_equation_without_real_roots() {
        let roots = QuadraticEquation::solve(1i32, 0, 1);

        assert!(roots.is_none());
    }
}
