use i_overlay::i_float::float::number::FloatNumber;

pub(crate) struct QuadraticEquation;

impl QuadraticEquation {
    pub(crate) fn solve<T: FloatNumber>(a: T, b: T, c: T) -> Option<[T; 2]> {
        if a == T::ZERO {
            return if b == T::ZERO {
                None
            } else {
                let t = -c / b;
                Some([t, t])
            };
        }

        let d = b * b - T::from_float(4.0) * a * c;
        if d < T::ZERO {
            return None;
        }

        let two_a = T::TWO * a;
        if d == T::ZERO {
            let t = -b / two_a;
            return Some([t, t]);
        }

        let sqrt_d = d.sqrt();
        Some([(-b - sqrt_d) / two_a, (-b + sqrt_d) / two_a])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_two_roots() {
        let roots = QuadraticEquation::solve(1.0, -3.0, 2.0).unwrap();

        assert_eq!(roots, [1.0, 2.0]);
    }

    #[test]
    fn solves_single_root() {
        let roots = QuadraticEquation::solve(1.0, -2.0, 1.0).unwrap();

        assert_eq!(roots, [1.0, 1.0]);
    }

    #[test]
    fn solves_linear_equation() {
        let roots = QuadraticEquation::solve(0.0, 2.0, -4.0).unwrap();

        assert_eq!(roots, [2.0, 2.0]);
    }

    #[test]
    fn ignores_equation_without_real_roots() {
        let roots = QuadraticEquation::solve(1.0, 0.0, 1.0);

        assert!(roots.is_none());
    }
}
