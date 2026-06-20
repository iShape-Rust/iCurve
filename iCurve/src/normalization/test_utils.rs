use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

pub(crate) fn assert_control_points_eq<T, P, const N: usize>(actual: [FloatPoint<T>; N], expected: [P; N])
where
    T: FloatNumber,
    P: FloatPointCompatible<Scalar = T>,
{
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert_point_eq(actual, expected);
    }
}

pub(crate) fn assert_point_eq<T, P>(actual: FloatPoint<T>, expected: P)
where
    T: FloatNumber,
    P: FloatPointCompatible<Scalar = T>,
{
    assert!((actual.x.to_f64() - expected.x().to_f64()).abs() < 0.000001);
    assert!((actual.y.to_f64() - expected.y().to_f64()).abs() < 0.000001);
}
