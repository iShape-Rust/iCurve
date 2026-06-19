use crate::kernel::cross::overlap::find::{CurveOverlap, FindOverlap, find_bezier_overlap};
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> FindOverlap<T> for QuadSegment<T> {
    fn find_overlap(&self, other: &Self, epsilon: T) -> Option<CurveOverlap<T>> {
        find_bezier_overlap(self, other, epsilon, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::curve::reversed::Reversed;
    use crate::kernel::curve::split_at::SplitAt;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.000001
    }

    #[test]
    fn finds_same_quad_overlap() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };

        let overlap = quad.find_overlap(&quad, 0.000001).unwrap();

        assert!(close(overlap.point_0.t0.value(), 0.0));
        assert!(close(overlap.point_0.t1.value(), 0.0));
        assert!(close(overlap.point_1.t0.value(), 1.0));
        assert!(close(overlap.point_1.t1.value(), 1.0));
    }

    #[test]
    fn finds_partial_quad_overlap() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };
        let right = quad.split_at_right(0.25);
        let sub_quad = right.split_at_left(2.0 / 3.0);

        let overlap = quad.find_overlap(&sub_quad, 0.000001).unwrap();

        assert!(close(overlap.point_0.t0.value(), 0.25));
        assert!(close(overlap.point_0.t1.value(), 0.0));
        assert!(close(overlap.point_1.t0.value(), 0.75));
        assert!(close(overlap.point_1.t1.value(), 1.0));
    }

    #[test]
    fn finds_reversed_quad_overlap() {
        let quad = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };
        let reversed = quad.reversed();

        let overlap = quad.find_overlap(&reversed, 0.000001).unwrap();

        assert!(close(overlap.point_0.t0.value(), 0.0));
        assert!(close(overlap.point_0.t1.value(), 1.0));
        assert!(close(overlap.point_1.t0.value(), 1.0));
        assert!(close(overlap.point_1.t1.value(), 0.0));
    }

    #[test]
    fn rejects_quad_with_same_endpoints_and_different_shape() {
        let quad_0 = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 2.0].into(), [2.0, 0.0].into()],
        };
        let quad_1 = QuadSegment {
            control_points: [[0.0, 0.0].into(), [1.0, -2.0].into(), [2.0, 0.0].into()],
        };

        assert!(quad_0.find_overlap(&quad_1, 0.000001).is_none());
    }
}
