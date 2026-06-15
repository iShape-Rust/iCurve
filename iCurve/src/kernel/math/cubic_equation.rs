use crate::kernel::math::quadratic_equation::QuadraticEquation;
use i_overlay::i_float::float::number::FloatNumber;

pub(crate) struct CubicEquation;

impl CubicEquation {
    pub(crate) fn solve_in_unit<T: FloatNumber>(a: T, b: T, c: T, d: T) -> [Option<T>; 3] {
        if a == T::ZERO {
            return Self::quadratic_in_unit(b, c, d);
        }

        let mut points = [T::ZERO; 4];
        let mut count = 0;
        Self::push_sorted_unique(&mut points, &mut count, T::ZERO);

        let three = T::from_float(3.0);
        if let Some(roots) = QuadraticEquation::solve(three * a, T::TWO * b, c) {
            Self::push_unit_sorted_unique(&mut points, &mut count, roots[0]);
            Self::push_unit_sorted_unique(&mut points, &mut count, roots[1]);
        }

        Self::push_sorted_unique(&mut points, &mut count, T::ONE);

        let mut roots = [None; 3];
        let mut root_count = 0;
        for i in 0..count {
            let t = points[i];
            if Self::value(a, b, c, d, t) == T::ZERO {
                Self::push_root(&mut roots, &mut root_count, t);
            }
        }

        for i in 1..count {
            let t0 = points[i - 1];
            let t1 = points[i];
            let v0 = Self::value(a, b, c, d, t0);
            let v1 = Self::value(a, b, c, d, t1);
            if v0 == T::ZERO || v1 == T::ZERO || v0.signum() == v1.signum() {
                continue;
            }

            let root = Self::bisect(a, b, c, d, t0, t1, v0);
            Self::push_root(&mut roots, &mut root_count, root);
        }

        roots
    }

    fn quadratic_in_unit<T: FloatNumber>(a: T, b: T, c: T) -> [Option<T>; 3] {
        let mut roots = [None; 3];
        let mut count = 0;

        if let Some(quadratic_roots) = QuadraticEquation::solve(a, b, c) {
            Self::push_unit_root(&mut roots, &mut count, quadratic_roots[0]);
            Self::push_unit_root(&mut roots, &mut count, quadratic_roots[1]);
        }

        roots
    }

    #[inline(always)]
    fn value<T: FloatNumber>(a: T, b: T, c: T, d: T, t: T) -> T {
        ((a * t + b) * t + c) * t + d
    }

    fn bisect<T: FloatNumber>(a: T, b: T, c: T, d: T, mut t0: T, mut t1: T, mut v0: T) -> T {
        let half = T::from_float(0.5);
        for _ in 0..T::BITS {
            let tm = (t0 + t1) * half;
            let vm = Self::value(a, b, c, d, tm);
            if vm == T::ZERO {
                return tm;
            }
            if vm.signum() == v0.signum() {
                t0 = tm;
                v0 = vm;
            } else {
                t1 = tm;
            }
        }

        (t0 + t1) * half
    }

    fn push_unit_sorted_unique<T: FloatNumber>(points: &mut [T; 4], count: &mut usize, t: T) {
        if t > T::ZERO && t < T::ONE {
            Self::push_sorted_unique(points, count, t);
        }
    }

    fn push_sorted_unique<T: FloatNumber>(points: &mut [T; 4], count: &mut usize, t: T) {
        for point in points.iter().take(*count) {
            if *point == t {
                return;
            }
        }

        let mut index = *count;
        while index > 0 && points[index - 1] > t {
            points[index] = points[index - 1];
            index -= 1;
        }

        points[index] = t;
        *count += 1;
    }

    fn push_unit_root<T: FloatNumber>(roots: &mut [Option<T>; 3], count: &mut usize, t: T) {
        if t >= T::ZERO && t <= T::ONE {
            Self::push_root(roots, count, t);
        }
    }

    fn push_root<T: FloatNumber>(roots: &mut [Option<T>; 3], count: &mut usize, t: T) {
        if *count == roots.len() {
            return;
        }

        for root in roots.iter().take(*count).flatten() {
            if *root == t {
                return;
            }
        }

        let mut index = *count;
        while index > 0 {
            let Some(prev) = roots[index - 1] else {
                break;
            };
            if prev <= t {
                break;
            }
            roots[index] = Some(prev);
            index -= 1;
        }

        roots[index] = Some(t);
        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots(values: [Option<f64>; 3]) -> [f64; 3] {
        [
            values[0].unwrap_or(f64::NAN),
            values[1].unwrap_or(f64::NAN),
            values[2].unwrap_or(f64::NAN),
        ]
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000001
    }

    #[test]
    fn solves_single_root_in_unit() {
        let roots = roots(CubicEquation::solve_in_unit(1.0, -1.5, 0.75, -0.125));

        assert!(close(roots[0], 0.5));
        assert!(roots[1].is_nan());
        assert!(roots[2].is_nan());
    }

    #[test]
    fn solves_three_roots_in_unit() {
        let roots = roots(CubicEquation::solve_in_unit(1.0, -1.5, 0.5, 0.0));

        assert!(close(roots[0], 0.0));
        assert!(close(roots[1], 0.5));
        assert!(close(roots[2], 1.0));
    }

    #[test]
    fn solves_degenerated_quadratic() {
        let roots = roots(CubicEquation::solve_in_unit(0.0, 1.0, -3.0, 2.0));

        assert!(close(roots[0], 1.0));
        assert!(roots[1].is_nan());
        assert!(roots[2].is_nan());
    }
}
