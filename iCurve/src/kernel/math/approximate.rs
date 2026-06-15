use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

pub(crate) struct ApproximateMath;

impl ApproximateMath {
    pub(crate) fn segment_contains<P: FloatPointCompatible>(
        a: P,
        b: P,
        p: P,
        relative_distance_epsilon: P::Scalar,
    ) -> bool {
        let a = FloatPoint::from_point(a);
        let b = FloatPoint::from_point(b);
        let p = FloatPoint::from_point(p);
        let ap = p - a;
        let bp = p - b;
        let ab = b - a;

        if bp.dot_product(ab).signum() == ap.dot_product(ab).signum() {
            return false;
        }

        // dist = ap × bp / |ab|
        // dist must be < epsilon * |ab|

        let cross = ap.cross_product(bp).abs();
        let ab_sqr_len = ab.sqr_length();

        cross < relative_distance_epsilon * ab_sqr_len
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::math::approximate::ApproximateMath;

    #[test]
    fn test_0() {
        assert!(ApproximateMath::segment_contains(
            [0.0, 0.0],
            [2.0, 0.0],
            [1.0, 0.0],
            0.0001
        ));
        assert!(!ApproximateMath::segment_contains(
            [0.0, 0.0],
            [2.0, 0.0],
            [1.0, 0.01],
            0.0001
        ));
        assert!(!ApproximateMath::segment_contains(
            [0.0, 0.0],
            [2.0, 0.0],
            [1.0, -0.01],
            0.0001
        ));
        assert!(ApproximateMath::segment_contains(
            [0.0, 0.0],
            [2.0, 0.0],
            [1.0, 0.000001],
            0.0001
        ));
        assert!(!ApproximateMath::segment_contains(
            [0.0, 0.0],
            [2.0, 0.0],
            [3.0, 0.0],
            0.0001
        ));
        assert!(!ApproximateMath::segment_contains(
            [0.0, 0.0],
            [2.0, 0.0],
            [-1.0, 0.0],
            0.0001
        ));
    }
}
