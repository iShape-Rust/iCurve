use crate::float::math::point::Point;
use crate::int::bezier::point::IntSplinePoints;
use crate::int::bezier::spline::IntBezierSplineApi;

pub(crate) trait IntSplineLength {
    fn avg_length(&self, min_cos: u32, min_len: u32) -> u128;
}

impl<Spline: IntBezierSplineApi> IntSplineLength for Spline {
    fn avg_length(&self, min_cos: u32, min_len: u32) -> u128 {
        let points = self.approximate_points(min_cos, min_len);
        let mut len = 0f64;
        for w in points.windows(2) {
            let a: Point = w[0].into();
            let b: Point = w[1].into();
            len += (a - b).length()
        }

        len as u128
    }
}

#[cfg(test)]
mod tests {
    use crate::int::bezier::length::IntSplineLength;
    use crate::int::bezier::spline_cube::IntCubeSpline;
    use crate::int::bezier::spline_line::IntLineSpline;
    use crate::int::bezier::spline_square::IntSquareSpline;
    use crate::int::math::normalize::VectorNormalization16Util;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_00() {
        let spline = IntLineSpline {
            anchors: [
                IntPoint::new(0, 0),
                IntPoint::new(100, 100),
            ],
        };

        assert_eq!(spline.avg_length(VectorNormalization16Util::normalize_unit_value(0.8), 3) as usize, 141);
    }

    #[test]
    fn test_01() {
        let spline = IntSquareSpline {
            anchors: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 100),
                IntPoint::new(100, 100),
            ],
        };

        assert_eq!(spline.avg_length(VectorNormalization16Util::normalize_unit_value(0.8), 3) as usize, 161);
    }

    #[test]
    fn test_02() {
        let spline = IntCubeSpline {
            anchors: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 50),
                IntPoint::new(50, 100),
                IntPoint::new(100, 100),
            ],
        };

        assert_eq!(spline.avg_length(VectorNormalization16Util::normalize_unit_value(0.8), 3) as usize, 153);
    }
}
