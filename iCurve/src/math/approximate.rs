use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::number::FloatNumber;

pub(crate) struct ApproximateMath;

impl ApproximateMath {
    pub(crate) fn is_point_near_segment<P: FloatPointCompatible>(a: P, b: P, p: P, relative_epsilon: P::Scalar) -> bool {
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

        cross < relative_epsilon * ab_sqr_len
    }
}

#[cfg(test)]
mod tests {
    use crate::math::approximate::ApproximateMath;

    #[test]
    fn test_0() {
        assert!(ApproximateMath::is_point_near_segment([0.0, 0.0], [2.0, 0.0], [1.0, 0.0], 0.0001));
        assert!(!ApproximateMath::is_point_near_segment([0.0, 0.0], [2.0, 0.0], [1.0, 0.01], 0.0001));
        assert!(!ApproximateMath::is_point_near_segment([0.0, 0.0], [2.0, 0.0], [1.0, -0.01], 0.0001));
        assert!(ApproximateMath::is_point_near_segment([0.0, 0.0], [2.0, 0.0], [1.0, 0.000001], 0.0001));
        assert!(!ApproximateMath::is_point_near_segment([0.0, 0.0], [2.0, 0.0], [3.0, 0.0], 0.0001));
        assert!(!ApproximateMath::is_point_near_segment([0.0, 0.0], [2.0, 0.0], [-1.0, 0.0], 0.0001));
    }
}