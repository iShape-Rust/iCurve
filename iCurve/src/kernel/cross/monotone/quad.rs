use crate::collections::stack_vec::StackVec;
use crate::kernel::cross::monotone::decomposite::{DecompositeIntoMonotone, MonotoneDecompositionDirection};
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;

#[derive(Debug, Clone, Copy)]
pub struct QuadMonotone<T: FloatNumber> {
    pub(crate) abc: [T; 3],
    pub(crate) t0: SegmentParam<T>,
    pub(crate) t1: SegmentParam<T>,
}

impl<T: FloatNumber> Default for QuadMonotone<T> {
    #[inline]
    fn default() -> Self {
        Self {
            abc: [T::ZERO; 3],
            t0: SegmentParam::Start,
            t1: SegmentParam::Start,
        }
    }
}

impl<T: FloatNumber> DecompositeIntoMonotone for QuadSegment<T> {
    type Output = StackVec<QuadMonotone<T>, 2>;

    fn decomposite_into_monotone(&self, direction: MonotoneDecompositionDirection) -> Self::Output {
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

        let mut result = StackVec::new();
        let derivative_a = T::TWO * a;

        if derivative_a != T::ZERO {
            let t = -b / derivative_a;

            if t > T::ZERO && t < T::ONE {
                let t = SegmentParam::Inner(t);
                result.push(QuadMonotone {
                    abc,
                    t0: SegmentParam::Start,
                    t1: t,
                });
                result.push(QuadMonotone {
                    abc,
                    t0: t,
                    t1: SegmentParam::End,
                });

                return result;
            }
        }

        result.push(QuadMonotone {
            abc,
            t0: SegmentParam::Start,
            t1: SegmentParam::End,
        });

        result
    }
}

impl<T: FloatNumber> QuadSegment<T> {
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
    use crate::kernel::math::rect::ToRect;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000001
    }

    #[test]
    fn splits_quad_by_axis_extremum() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };

        let parts = quad.decomposite_into_monotone(MonotoneDecompositionDirection::Y);

        let parts = parts.as_slice();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].t0, SegmentParam::Start);
        assert_eq!(parts[0].t1, SegmentParam::Inner(0.5));
        assert_eq!(parts[1].t0, SegmentParam::Inner(0.5));
        assert_eq!(parts[1].t1, SegmentParam::End);
    }

    #[test]
    fn keeps_monotone_quad_as_single_part() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 1.0].into(), [2.0, 2.0].into()],
        };

        let parts = quad.decomposite_into_monotone(MonotoneDecompositionDirection::with_rect(quad.to_rect()));

        let parts = parts.as_slice();

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].t0, SegmentParam::Start);
        assert_eq!(parts[0].t1, SegmentParam::End);
    }

    #[test]
    fn stores_axis_power_coefficients() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 1.0].into(), [2.0, 2.0].into()],
        };

        let parts = quad.decomposite_into_monotone(MonotoneDecompositionDirection::with_rect(quad.to_rect()));
        let part = parts.as_slice()[0];

        assert!(close(part.abc[0], 0.0));
        assert!(close(part.abc[1], 2.0));
        assert!(close(part.abc[2], 0.0));
    }
}
