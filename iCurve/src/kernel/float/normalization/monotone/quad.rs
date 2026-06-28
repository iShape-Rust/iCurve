use crate::collections::stack_vec::StackVec;
use crate::kernel::float::curve::param::FloatSegmentParam;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use crate::kernel::float::normalization::monotone::decomposite::{
    DecompositeIntoMonotone, MonotoneDecompositionDirection, roots_to_segments,
};
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> DecompositeIntoMonotone for FloatQuadSegment<T> {
    type Output = StackVec<FloatQuadSegment<T>, 3>;

    fn decomposite_into_monotone(&self) -> Self::Output {
        let x_root = self.monotone_roots_by_direction(MonotoneDecompositionDirection::X);
        let y_root = self.monotone_roots_by_direction(MonotoneDecompositionDirection::Y);
        let mut roots = StackVec::<_, 2>::new();
        roots.push_some(x_root);
        roots.push_some(y_root);

        roots_to_segments(self, roots)
    }
}

impl<T: FloatNumber> FloatQuadSegment<T> {
    fn monotone_roots_by_direction(
        &self,
        direction: MonotoneDecompositionDirection,
    ) -> Option<FloatSegmentParam<T>> {
        let abc = Self::axis_abc(self, direction);

        // Quadratic Bezier in Bernstein form:
        //
        //   P(t) = (1 - t)^2 * P0
        //        + 2 * (1 - t) * t * P1
        //        + t^2 * P2
        //
        // For the selected axis we convert it to power form:
        //
        //   q(t) = a * t^2 + b * t + c
        //
        // A quadratic is monotone on every interval that does not contain an
        // extremum of q(t). Extrema happen where the derivative is zero:
        //
        //   q'(t) = 2 * a * t + b = 0
        //
        // The derivative is linear, so it has at most one root. That root
        // splits [0, 1] into at most two monotone intervals.
        let [a, b, _] = abc;

        let derivative_a = T::TWO * a;

        if derivative_a != T::ZERO {
            let t = -b / derivative_a;
            if t > T::ZERO && t < T::ONE {
                return Some(FloatSegmentParam::new(t));
            }
        }

        None
    }

    #[inline]
    fn axis_abc(&self, direction: MonotoneDecompositionDirection) -> [T; 3] {
        let [k0, k1, k2] = match direction {
            MonotoneDecompositionDirection::X => self.control_points.map(|p| p.x),
            MonotoneDecompositionDirection::Y => self.control_points.map(|p| p.y),
        };

        // Expanding the Bernstein quadratic gives:
        //
        //   q(t) = a * t^2 + b * t + c
        //
        // where:
        //
        //   a = k2 - 2 * k1 + k0
        //   b = 2 * (k1 - k0)
        //   c = k0
        let two = T::TWO;

        [k2 - k1 * two + k0, (k1 - k0) * two, k0]
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
    fn splits_quad_by_both_axis_extrema() {
        let quad = FloatQuadSegment {
            control_points: [[0.0, 0.0].into(), [2.0, 1.0].into(), [0.0, -2.0].into()],
        };

        let parts = quad.decomposite_into_monotone();
        let parts = parts.as_slice();

        assert_eq!(parts.len(), 3);
        assert_points_close(
            parts[0].control_points[0],
            quad.point_at(FloatSegmentParam::Start),
        );
        assert_points_close(
            parts[0].control_points[2],
            quad.point_at(FloatSegmentParam::Inner(0.25)),
        );
        assert_points_close(
            parts[1].control_points[2],
            quad.point_at(FloatSegmentParam::Inner(0.5)),
        );
        assert_points_close(parts[2].control_points[2], quad.point_at(FloatSegmentParam::End));
    }
}
