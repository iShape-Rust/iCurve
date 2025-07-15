use crate::float::math::point::Point;
use crate::int::bezier::point::IntSplinePoints;
use crate::int::bezier::spline::IntCADSpline;

pub(crate) trait IntSplineLength {
    fn avg_length(&self, min_cos: u32, min_len: u32) -> u128;
}

impl<Spline: IntCADSpline> IntSplineLength for Spline {
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
    use crate::int::bezier::spline_cube::IntCubeSpline;
    use crate::int::bezier::spline_line::IntLineSpline;
    use crate::int::bezier::length::IntSplineLength;
    use crate::int::bezier::spline_square::IntSquareSpline;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_00() {
        let spline = IntLineSpline {
            a: IntPoint::new(0, 0),
            b: IntPoint::new(100, 100),
        };

        assert_eq!(spline.avg_length(800, 3) as usize, 141);
    }

    #[test]
    fn test_01() {
        let spline = IntSquareSpline {
            a: IntPoint::new(0, 0),
            m: IntPoint::new(0, 100),
            b: IntPoint::new(100, 100),
        };

        assert_eq!(spline.avg_length(800, 3) as usize, 161);
    }

    #[test]
    fn test_02() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 50),
            bm: IntPoint::new(50, 100),
            b: IntPoint::new(100, 100),
        };

        assert_eq!(spline.avg_length(800, 3) as usize, 153);
    }
}