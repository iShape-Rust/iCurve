use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::math::quadratic_equation::QuadraticEquation;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

impl<T: FloatNumber> QuadSegment<T> {
    pub fn contains(&self, p: FloatPoint<T>, eps: T) -> Option<T> {
        // P(t) = (1 - t)^2 P0 + 2(1 - t)t P1 + t^2 P2
        let [p0, p1, p2] = self.control_points;

        let a = p2 - p1 * T::TWO + p0;
        let b = (p1 - p0) * T::TWO;
        let c = p0 - p;

        // P = a*t^2 + b*t + c

        let eps_sqr = eps * eps;
        debug_assert!(
            a.sqr_length() != T::ZERO || b.sqr_length() != T::ZERO,
            "degenerate quad segment is not supported"
        );

        let mut best = None;

        if let Some(roots) = QuadraticEquation::solve(a.x, b.x, c.x) {
            Self::update_best(p, a, b, p0, eps_sqr, roots[0], &mut best);
            Self::update_best(p, a, b, p0, eps_sqr, roots[1], &mut best);
        }

        if let Some(roots) = QuadraticEquation::solve(a.y, b.y, c.y) {
            Self::update_best(p, a, b, p0, eps_sqr, roots[0], &mut best);
            Self::update_best(p, a, b, p0, eps_sqr, roots[1], &mut best);
        }

        best.map(|(t, _)| t)
    }

    fn update_best(
        target: FloatPoint<T>,
        a: FloatPoint<T>,
        b: FloatPoint<T>,
        p0: FloatPoint<T>,
        eps_sqr: T,
        t: T,
        best: &mut Option<(T, T)>,
    ) {
        let zero = T::ZERO;
        let one = T::ONE;
        if !(t >= zero && t <= one) {
            return;
        }

        let point = a * (t * t) + b * t + p0;
        let sqr_distance = (point - target).sqr_length();
        if !(sqr_distance <= eps_sqr) {
            return;
        }

        match best {
            Some((_, best_sqr_distance)) if *best_sqr_distance <= sqr_distance => {}
            _ => *best = Some((t, sqr_distance)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000001
    }

    #[test]
    fn finds_parameter_on_quad() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };

        let t = quad.contains([1.0, 1.0].into(), 0.000001).unwrap();

        assert!(close(t, 0.5));
    }

    #[test]
    fn finds_end_parameter() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };

        let t = quad.contains([2.0, 0.0].into(), 0.000001).unwrap();

        assert!(close(t, 1.0));
    }

    #[test]
    fn ignores_point_outside_epsilon() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };

        assert!(quad.contains([1.0, 1.1].into(), 0.000001).is_none());
    }
}
