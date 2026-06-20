use crate::collections::stack_vec::StackVec;
use crate::kernel::cross::monotone::decomposite::{DecompositeIntoMonotone, MonotoneDecompositionDirection};
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::math::quadratic_equation::QuadraticEquation;
use i_overlay::i_float::float::number::FloatNumber;

#[derive(Debug, Clone, Copy)]
pub struct CubicMonotone<T: FloatNumber> {
    pub(crate) abcd: [T; 4],
    pub(crate) t0: SegmentParam<T>,
    pub(crate) t1: SegmentParam<T>,
}

impl<T: FloatNumber> Default for CubicMonotone<T> {
    #[inline]
    fn default() -> Self {
        Self {
            abcd: [T::ZERO; 4],
            t0: SegmentParam::Start,
            t1: SegmentParam::Start,
        }
    }
}

impl<T: FloatNumber> DecompositeIntoMonotone for CubicSegment<T> {
    type Output = StackVec<CubicMonotone<T>, 3>;

    fn decomposite_into_monotone(&self, direction: MonotoneDecompositionDirection) -> Self::Output {
        let abcd = Self::axis_abcd(self, direction);

        // Cubic Bezier in Bernstein form:
        //
        //   P(t) = (1 - t)^3 * P0
        //        + 3 * (1 - t)^2 * t * P1
        //        + 3 * (1 - t) * t^2 * P2
        //        + t^3 * P3
        //
        // For the selected axis we convert it to power form:
        //
        //   q(t) = a * t^3 + b * t^2 + c * t + d
        //
        // A cubic is monotone on every interval that does not contain an
        // extremum of q(t). Extrema happen where the derivative is zero:
        //
        //   q'(t) = 3 * a * t^2 + 2 * b * t + c = 0
        //
        // The derivative is quadratic, so it has at most two roots. Those
        // roots slice [0, 1] into at most three monotone intervals.
        let [a, b, c, _] = abcd;
        let mut roots = StackVec::new();

        if let Some(derivative_roots) = QuadraticEquation::solve(T::THREE * a, T::TWO * b, c) {
            Self::push_inner_root(&mut roots, derivative_roots[0]);
            Self::push_inner_root(&mut roots, derivative_roots[1]);
        }

        let mut result = StackVec::new();
        let mut t0 = SegmentParam::Start;

        for &t1 in roots.as_slice() {
            result.push(CubicMonotone { abcd, t0, t1 });
            t0 = t1;
        }

        result.push(CubicMonotone {
            abcd,
            t0,
            t1: SegmentParam::End,
        });

        result
    }
}

impl<T: FloatNumber> CubicSegment<T> {
    #[inline]
    fn axis_abcd(&self, direction: MonotoneDecompositionDirection) -> [T; 4] {
        let [k0, k1, k2, k3] = match direction {
            MonotoneDecompositionDirection::X => self.control_points.map(|p| p.x),
            MonotoneDecompositionDirection::Y => self.control_points.map(|p| p.y),
        };

        // Expanding the Bernstein cubic gives:
        //
        //   q(t) = a * t^3 + b * t^2 + c * t + d
        //
        // where:
        //
        //   a = k3 - 3 * k2 + 3 * k1 - k0
        //   b = 3 * k0 - 6 * k1 + 3 * k2
        //   c = 3 * (k1 - k0)
        //   d = k0
        let three = T::THREE;
        let six = T::from_float(6.0);

        [
            k3 - k2 * three + k1 * three - k0,
            k0 * three - k1 * six + k2 * three,
            (k1 - k0) * three,
            k0,
        ]
    }
    fn push_inner_root(roots: &mut StackVec<SegmentParam<T>, 2>, t: T) {
        if !(t > T::ZERO && t < T::ONE) {
            return;
        }

        let root = SegmentParam::Inner(t);

        for existing in roots.as_slice() {
            if *existing == root {
                return;
            }
        }

        if roots.is_empty() || roots.as_slice()[roots.len() - 1] < root {
            roots.push(root);
            return;
        }

        let previous = roots.swap_extract(0);
        roots.push(root);
        roots.push(previous);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::math::rect::ToRect;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000001
    }

    #[test]
    fn splits_cubic_by_two_axis_extrema() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 3.0].into(),
                [2.0, -3.0].into(),
                [3.0, 0.0].into(),
            ],
        };

        let parts = cubic.decomposite_into_monotone(MonotoneDecompositionDirection::Y);

        let parts = parts.as_slice();

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].t0, SegmentParam::Start);
        assert!(matches!(parts[0].t1, SegmentParam::Inner(_)));
        assert_eq!(parts[2].t1, SegmentParam::End);
    }

    #[test]
    fn keeps_monotone_cubic_as_single_part() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 1.0].into(),
                [2.0, 2.0].into(),
                [3.0, 3.0].into(),
            ],
        };

        let parts =
            cubic.decomposite_into_monotone(MonotoneDecompositionDirection::with_rect(cubic.to_rect()));

        let parts = parts.as_slice();

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].t0, SegmentParam::Start);
        assert_eq!(parts[0].t1, SegmentParam::End);
    }

    #[test]
    fn stores_axis_power_coefficients() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 1.0].into(),
                [2.0, 2.0].into(),
                [3.0, 3.0].into(),
            ],
        };

        let parts =
            cubic.decomposite_into_monotone(MonotoneDecompositionDirection::with_rect(cubic.to_rect()));
        let part = parts.as_slice()[0];

        assert!(close(part.abcd[0], 0.0));
        assert!(close(part.abcd[1], 0.0));
        assert!(close(part.abcd[2], 3.0));
        assert!(close(part.abcd[3], 0.0));
    }
}
