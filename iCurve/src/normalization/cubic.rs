use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::point_at::PointAt;
use crate::kernel::math::quadratic_equation::QuadraticEquation;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

impl<T: FloatNumber> CubicSegment<T> {
    pub(super) fn resolve_self_intersection(&self) -> Option<CubicSelfIntersection<T>> {
        let [p0, p1, p2, p3] = self.control_points;

        // Cubic Bezier in Bernstein form:
        //
        //   P(t) = (1 - t)^3 * P0
        //        + 3 * (1 - t)^2 * t * P1
        //        + 3 * (1 - t) * t^2 * P2
        //        + t^3 * P3
        //
        // Expanding it to power form gives:
        //
        //   P(t) = a * t^3 + b * t^2 + c * t + P0
        //
        // A self-intersection has two distinct parameters u and v where:
        //
        //   P(u) = P(v)
        //
        // After factoring P(u) - P(v) by (u - v), and denoting:
        //
        //   s = u + v
        //   p = u * v
        //
        // we get the vector equation:
        //
        //   a * (s^2 - p) + b * s + c = 0
        //
        // Taking the cross product with a removes the a term:
        //
        //   s = -cross(a, c) / cross(a, b)
        //
        // Then p is the projection of q onto a, where:
        //
        //   q = a * s^2 + b * s + c = a * p
        //
        // Finally u and v are the roots of:
        //
        //   t^2 - s * t + p = 0
        let a = p3 - p2 * T::THREE + p1 * T::THREE - p0;
        let b = (p2 - p1 * T::TWO + p0) * T::THREE;
        let c = (p1 - p0) * T::THREE;

        let ab = a.cross_product(b);
        if ab == T::ZERO {
            return None;
        }

        let aa = a.sqr_length();
        if aa == T::ZERO {
            return None;
        }

        let s = -a.cross_product(c) / ab;
        let q = a * (s * s) + b * s + c;
        let p = a.dot_product(q) / aa;

        let [t0, t1] = QuadraticEquation::solve(T::ONE, -s, p)?;
        if !(T::ZERO < t0 && t0 < T::ONE && T::ZERO < t1 && t1 < T::ONE && t0 != t1) {
            return None;
        }

        Some(CubicSelfIntersection {
            t0,
            t1,
            point: self.point_at(SegmentParam::Inner(t0)),
        })
    }

}

pub(crate) struct CubicSelfIntersection<T: FloatNumber> {
    pub(crate) t0: T,
    pub(crate) t1: T,
    pub(crate) point: FloatPoint<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_cubic_self_intersection() {
        let intersection = CubicSegment {
            control_points: [
                [0.0f64, 0.0].into(),
                [-3.0, -3.0].into(),
                [-3.0, -2.0].into(),
                [-2.0, -2.0].into(),
            ],
        }
        .resolve_self_intersection()
        .unwrap();

        assert!((intersection.t0.to_f64() - 3.0 / 7.0).abs() < 0.000001);
        assert!((intersection.t1.to_f64() - 6.0 / 7.0).abs() < 0.000001);
        assert!((intersection.point.x.to_f64() + 2.3615160349854225).abs() < 0.000001);
        assert!((intersection.point.y.to_f64() + 2.0466472303206995).abs() < 0.000001);
    }

    #[test]
    fn ignores_non_intersecting_cubic() {
        let intersection = CubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [1.0, 2.0].into(),
                [3.0, 2.0].into(),
                [4.0, 0.0].into(),
            ],
        }
        .resolve_self_intersection();

        assert!(intersection.is_none());
    }
}
