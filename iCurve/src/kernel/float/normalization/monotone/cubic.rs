use crate::collections::stack_vec::StackVec;
use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::param::FloatSegmentParam;
use crate::kernel::float::math::quadratic_equation::QuadraticEquation;
use crate::kernel::float::normalization::monotone::decomposite::{
    DecompositeIntoMonotone, MonotoneDecompositionDirection, roots_to_segments,
};
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> DecompositeIntoMonotone for FloatCubicSegment<T> {
    type Output = StackVec<FloatCubicSegment<T>, 5>;

    fn decomposite_into_monotone(&self) -> Self::Output {
        let x_roots = self.monotone_roots_by_direction(MonotoneDecompositionDirection::X);
        let y_roots = self.monotone_roots_by_direction(MonotoneDecompositionDirection::Y);

        let mut roots = StackVec::<_, 4>::new();

        roots.extend_from_slice(x_roots.as_slice());
        roots.extend_from_slice(y_roots.as_slice());

        roots_to_segments(self, roots)
    }
}

impl<T: FloatNumber> FloatCubicSegment<T> {
    fn monotone_roots_by_direction(
        &self,
        direction: MonotoneDecompositionDirection,
    ) -> StackVec<FloatSegmentParam<T>, 2> {
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
        let mut roots = StackVec::<FloatSegmentParam<T>, 2>::new();

        if let Some(derivative_roots) = QuadraticEquation::solve(T::THREE * a, T::TWO * b, c) {
            roots.push_some(FloatSegmentParam::inner(derivative_roots[0]));
            roots.push_some(FloatSegmentParam::inner(derivative_roots[1]));
        }

        roots
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::float::curve::point_at::FloatPointAt;
    use i_overlay::i_float::float::point::FloatPoint;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000001
    }

    fn assert_points_close(a: FloatPoint<f64>, b: FloatPoint<f64>) {
        assert!(close(a.x, b.x));
        assert!(close(a.y, b.y));
    }

    #[test]
    fn splits_cubic_by_both_directions() {
        let cubic = FloatCubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 3.0].into(),
                [2.0, -3.0].into(),
                [3.0, 0.0].into(),
            ],
        };

        let parts = cubic.decomposite_into_monotone();
        let parts = parts.as_slice();

        assert_eq!(parts.len(), 3);
        assert_points_close(
            parts[0].control_points[0],
            cubic.point_at(FloatSegmentParam::Start),
        );
        assert_points_close(parts[2].control_points[3], cubic.point_at(FloatSegmentParam::End));
    }
}
