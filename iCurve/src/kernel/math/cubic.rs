use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::math::cubic_equation::CubicEquation;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

impl<T: FloatNumber> CubicSegment<T> {
    pub fn contains(&self, p: FloatPoint<T>, eps: T) -> Option<T> {
        // P(t) = (1 - t)^3 P0 + 3(1 - t)^2 t P1 + 3(1 - t) t^2 P2 + t^3 P3

        let [p0, p1, p2, p3] = self.control_points;

        let three = T::THREE;
        let six = T::from_float(6.0);
        let a = p3 - p2 * three + p1 * three - p0;
        let b = p0 * three - p1 * six + p2 * three;
        let c = (p1 - p0) * three;
        let d = p0 - p;

        // P = a*t^3 + b*t^2 + c*t + d

        let eps_sqr = eps * eps;
        debug_assert!(
            a.sqr_length() != T::ZERO || b.sqr_length() != T::ZERO || c.sqr_length() != T::ZERO,
            "degenerate cubic segment is not supported"
        );

        let mut best = None;

        let x_roots = CubicEquation::solve_in_unit(a.x, b.x, c.x, d.x);
        for root in x_roots.iter().flatten() {
            Self::update_best(p, a, b, c, p0, eps_sqr, *root, &mut best);
        }

        let y_roots = CubicEquation::solve_in_unit(a.y, b.y, c.y, d.y);
        for root in y_roots.iter().flatten() {
            Self::update_best(p, a, b, c, p0, eps_sqr, *root, &mut best);
        }

        best.map(|(t, _)| t)
    }

    fn update_best(
        target: FloatPoint<T>,
        a: FloatPoint<T>,
        b: FloatPoint<T>,
        c: FloatPoint<T>,
        p0: FloatPoint<T>,
        eps_sqr: T,
        t: T,
        best: &mut Option<(T, T)>,
    ) {
        let point = ((a * t + b) * t + c) * t + p0;
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
    fn finds_parameter_on_cubic() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };

        let t = cubic.contains([2.0, 1.5].into(), 0.000001).unwrap();

        assert!(close(t, 0.5));
    }

    #[test]
    fn finds_cubic_end_parameter() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };

        let t = cubic.contains([4.0, 0.0].into(), 0.000001).unwrap();

        assert!(close(t, 1.0));
    }

    #[test]
    fn ignores_cubic_point_outside_epsilon() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };

        assert!(cubic.contains([2.0, 1.6].into(), 0.000001).is_none());
    }
}
