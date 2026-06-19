use crate::kernel::cross::overlap::find::{CurveOverlap, FindOverlap, find_bezier_overlap};
use crate::kernel::curve::cubic::CubicSegment;
use i_overlay::i_float::float::number::FloatNumber;

impl<T: FloatNumber> FindOverlap<T> for CubicSegment<T> {
    fn find_overlap(&self, other: &Self, epsilon: T) -> Option<CurveOverlap<T>> {
        find_bezier_overlap(self, other, epsilon, 2)
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
    fn finds_same_cubic_overlap() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };

        let overlap = cubic.find_overlap(&cubic, 0.000001).unwrap();

        assert!(close(overlap.point_0.t0.value(), 0.0));
        assert!(close(overlap.point_0.t1.value(), 0.0));
        assert!(close(overlap.point_1.t0.value(), 1.0));
        assert!(close(overlap.point_1.t1.value(), 1.0));
    }

    #[test]
    fn finds_partial_cubic_overlap() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };
        let right = cubic.split_at_right(0.25);
        let sub_cubic = right.split_at_left(2.0 / 3.0);

        let overlap = cubic.find_overlap(&sub_cubic, 0.000001).unwrap();

        assert!(close(overlap.point_0.t0.value(), 0.25));
        assert!(close(overlap.point_0.t1.value(), 0.0));
        assert!(close(overlap.point_1.t0.value(), 0.75));
        assert!(close(overlap.point_1.t1.value(), 1.0));
    }

    #[test]
    fn finds_reversed_cubic_overlap() {
        let cubic = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };
        let reversed = cubic.reversed();

        let overlap = cubic.find_overlap(&reversed, 0.000001).unwrap();

        assert!(close(overlap.point_0.t0.value(), 0.0));
        assert!(close(overlap.point_0.t1.value(), 1.0));
        assert!(close(overlap.point_1.t0.value(), 1.0));
        assert!(close(overlap.point_1.t1.value(), 0.0));
    }

    #[test]
    fn rejects_cubic_with_same_endpoints_and_different_shape() {
        let cubic_0 = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        };
        let cubic_1 = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, -2.0].into(),
                [3.0, -2.0].into(),
                [4.0, 0.0].into(),
            ],
        };

        assert!(cubic_0.find_overlap(&cubic_1, 0.000001).is_none());
    }
}
