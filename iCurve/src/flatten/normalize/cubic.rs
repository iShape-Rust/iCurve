use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::vector::FloatPointMath;

pub(crate) struct CubicSelfIntersection<T: FloatNumber> {
    pub(crate) t0: T,
    pub(crate) t1: T,
    pub(crate) point: FloatPoint<T>,
}

pub(crate) fn find_cubic_self_intersection<T: FloatNumber>(
    p0: FloatPoint<T>,
    p1: FloatPoint<T>,
    p2: FloatPoint<T>,
    p3: FloatPoint<T>,
) -> Option<CubicSelfIntersection<T>> {
    let a = FloatPointMath::add(
        &FloatPointMath::sub(&scale_float(p1, 3.0), &p0),
        &FloatPointMath::sub(&p3, &scale_float(p2, 3.0)),
    );
    let b = FloatPointMath::add(
        &FloatPointMath::sub(&scale_float(p0, 3.0), &scale_float(p1, 6.0)),
        &scale_float(p2, 3.0),
    );
    let c = scale_float(FloatPointMath::sub(&p1, &p0), 3.0);

    let zero = T::from_float(0.0);
    let one = T::from_float(1.0);
    let two = T::from_float(2.0);
    let four = T::from_float(4.0);

    let ab = FloatPointMath::cross_product(&a, &b);
    if ab == zero {
        return None;
    }

    let aa = FloatPointMath::sqr_length(&a);
    if aa == zero {
        return None;
    }

    let s = -FloatPointMath::cross_product(&a, &c) / ab;
    let q = FloatPointMath::add(
        &FloatPointMath::add(&FloatPointMath::scale(&a, s * s), &FloatPointMath::scale(&b, s)),
        &c,
    );
    let p = FloatPointMath::dot_product(&a, &q) / aa;
    let d = s * s - four * p;
    if d <= zero {
        return None;
    }

    let sqrt_d = d.sqrt();
    let t0 = (s - sqrt_d) / two;
    let t1 = (s + sqrt_d) / two;
    if !(zero < t0 && t0 < one && zero < t1 && t1 < one && t0 != t1) {
        return None;
    }

    Some(CubicSelfIntersection {
        t0,
        t1,
        point: cubic_point_at(p0, p1, p2, p3, t0),
    })
}

pub(crate) fn cubic_point_at<P: FloatPointCompatible>(p0: P, p1: P, p2: P, p3: P, t: P::Scalar) -> P {
    let p01 = point_at(p0, p1, t);
    let p12 = point_at(p1, p2, t);
    let p23 = point_at(p2, p3, t);
    let p012 = point_at(p01, p12, t);
    let p123 = point_at(p12, p23, t);
    point_at(p012, p123, t)
}

fn point_at<P: FloatPointCompatible>(a: P, b: P, t: P::Scalar) -> P {
    P::from_xy(a.x() + (b.x() - a.x()) * t, a.y() + (b.y() - a.y()) * t)
}

fn scale_float<P: FloatPointCompatible>(p: P, k: f64) -> P {
    let k = P::Scalar::from_float(k);
    FloatPointMath::scale(&p, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_cubic_self_intersection() {
        let intersection = find_cubic_self_intersection(
            [0.0f64, 0.0].into(),
            [-3.0, -3.0].into(),
            [-3.0, -2.0].into(),
            [-2.0, -2.0].into(),
        )
        .unwrap();

        assert!((intersection.t0.to_f64() - 3.0 / 7.0).abs() < 0.000001);
        assert!((intersection.t1.to_f64() - 6.0 / 7.0).abs() < 0.000001);
        assert!((intersection.point.x.to_f64() + 2.3615160349854225).abs() < 0.000001);
        assert!((intersection.point.y.to_f64() + 2.0466472303206995).abs() < 0.000001);
    }

    #[test]
    fn ignores_non_intersecting_cubic() {
        let intersection = find_cubic_self_intersection(
            [0.0, 0.0].into(),
            [1.0, 2.0].into(),
            [3.0, 2.0].into(),
            [4.0, 0.0].into(),
        );

        assert!(intersection.is_none());
    }
}
