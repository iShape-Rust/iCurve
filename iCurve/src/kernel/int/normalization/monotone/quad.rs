use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::normalization::monotone::decomposition::{
    DecomposeIntoMonotone, MonotoneDecompositionDirection, roots_to_segments,
};
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

impl<I: IntNumber> DecomposeIntoMonotone for QuadSegment<I> {
    type Output = StackVec<QuadSegment<I>, 3>;

    fn decompose_into_monotone(&self) -> Self::Output {
        let x_root = self.monotone_root_by_direction(MonotoneDecompositionDirection::X);
        let y_root = self.monotone_root_by_direction(MonotoneDecompositionDirection::Y);

        let mut roots = StackVec::<_, 2>::new();
        roots.push_some(x_root);
        roots.push_some(y_root);

        roots_to_segments(self, roots)
    }
}

impl<I: IntNumber> QuadSegment<I> {
    fn monotone_root_by_direction(
        &self,
        direction: MonotoneDecompositionDirection,
    ) -> Option<SegmentParam<I>> {
        let [k0, k1, k2] = Self::axis_values(self.control_points, direction);

        // Quadratic Bezier in Bernstein form:
        //
        //   q(t) = (1 - t)^2 * k0
        //        + 2 * (1 - t) * t * k1
        //        + t^2 * k2
        //
        // In power form:
        //
        //   q(t) = a * t^2 + b * t + k0
        //
        // where:
        //
        //   a = k2 - 2 * k1 + k0
        //   b = 2 * (k1 - k0)
        //
        // Extrema happen where:
        //
        //   q'(t) = 2 * a * t + b = 0
        //
        // The factor 2 cancels out, so:
        //
        //   t = (k0 - k1) / (k2 - 2 * k1 + k0)
        //
        // Keep this division in wide fixed-scale form. Integer division here
        // would drop roots such as 1/3 and skip required monotone splits.
        let numerator = k0 - k1;
        let denominator = k2 - I::Wide::TWO * k1 + k0;

        if denominator == I::Wide::ZERO {
            return None;
        }

        let t_scaled = FixedScale::<I>::div_to_scaled_round(numerator, denominator);
        // 0 < t < 1
        if t_scaled <= I::Wide::ZERO || t_scaled >= SegmentParam::<I>::DENOMINATOR {
            return None;
        }

        Some(SegmentParam::new(I::from_wide(t_scaled)))
    }

    #[inline]
    fn axis_values(
        control_points: [IntPoint<I>; 3],
        direction: MonotoneDecompositionDirection,
    ) -> [I::Wide; 3] {
        match direction {
            MonotoneDecompositionDirection::X => control_points.map(|p| p.x.to_wide()),
            MonotoneDecompositionDirection::Y => control_points.map(|p| p.y.to_wide()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::int::curve::point_at::PointAt;

    #[test]
    fn splits_quad_by_both_axis_extrema() {
        let quad = QuadSegment::<i32> {
            control_points: [IntPoint::new(0, 0), IntPoint::new(2, 1), IntPoint::new(0, -2)],
        };

        let parts = quad.decompose_into_monotone();
        let parts = parts.as_slice();

        assert_eq!(parts.len(), 3);
        assert_eq!(
            parts[0].control_points[0],
            quad.control_points.point_at(SegmentParam::new(0))
        );
        assert_eq!(
            parts[0].control_points[2],
            quad.control_points.point_at(SegmentParam::from_int(1, 4))
        );
        assert_eq!(
            parts[1].control_points[2],
            quad.control_points.point_at(SegmentParam::from_int(1, 2))
        );
        assert_eq!(
            parts[2].control_points[2],
            quad.control_points
                .point_at(SegmentParam::new(SegmentParam::<i32>::DENOMINATOR as i32))
        );
    }

    #[test]
    fn keeps_fractional_extremum_in_fixed_scale() {
        let quad = QuadSegment::<i32> {
            control_points: [IntPoint::new(0, 0), IntPoint::new(2, 3), IntPoint::new(-2, 0)],
        };

        let root = quad
            .monotone_root_by_direction(MonotoneDecompositionDirection::Y)
            .unwrap();

        assert_eq!(root.value(), SegmentParam::<i32>::from_int(1, 2).value());

        let x_root = quad
            .monotone_root_by_direction(MonotoneDecompositionDirection::X)
            .unwrap();

        assert!((x_root.value() - SegmentParam::<i32>::from_int(1, 3).value()).abs() <= 1i64);
    }
}
