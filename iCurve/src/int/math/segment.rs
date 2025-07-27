use crate::float::math::point::Point;
use crate::int::math::point::IntPoint;

pub(crate) struct IntSegment {
    pub(crate) a: IntPoint,
    pub(crate) b: IntPoint,
}

impl IntSegment {
    #[inline]
    pub(crate) fn new(a: IntPoint, b: IntPoint) -> Self {
        Self { a, b }
    }

    #[inline]
    pub(crate) fn closest_point(&self, p: IntPoint) -> IntPoint {
        let pa = p - self.a;
        let pb = p - self.b;
        let ba = self.b - self.a;

        let a_dot = pa.dot_product(&ba);
        let b_dot = pb.dot_product(&ba);

        let a_dir = a_dot > 0;
        let b_dir = b_dot > 0;

        if a_dir == b_dir {
            return if a_dir { self.b } else { self.a };
        }

        let v = Point::from_int(ba);
        let n = v.normalized();
        let pa = v.dot_product(&n);
        let na = (n * pa).to_round_int();

        self.a + na
    }
}

impl Point {
    #[inline]
    fn to_round_int(&self) -> IntPoint {
        let x = (self.x + 0.5_f64.copysign(self.x)) as i64;
        let y = (self.y + 0.5_f64.copysign(self.y)) as i64;
        IntPoint::new(x, y)
    }

    #[inline]
    fn from_int(point: IntPoint) -> Self {
        Self::new(point.x as f64, point.y as f64)
    }
}

#[cfg(test)]
mod tests {
    use crate::int::math::point::IntPoint;
    use crate::int::math::segment::IntSegment;

    #[test]
    fn test_0() {
        let s = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        assert_eq!(s.closest_point(IntPoint::new(-10, 10)), IntPoint::new(0, 0));
        assert_eq!(
            s.closest_point(IntPoint::new(-10, -10)),
            IntPoint::new(0, 0)
        );
    }

    #[test]
    fn test_1() {
        let s = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        assert_eq!(s.closest_point(IntPoint::new(20, 10)), IntPoint::new(10, 0));
        assert_eq!(
            s.closest_point(IntPoint::new(20, -10)),
            IntPoint::new(10, 0)
        );
    }

    #[test]
    fn test_2() {
        let s = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        assert_eq!(s.closest_point(IntPoint::new(0, 10)), IntPoint::new(0, 0));
        assert_eq!(s.closest_point(IntPoint::new(0, -10)), IntPoint::new(0, 0));
    }

    #[test]
    fn test_3() {
        let s = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        assert_eq!(s.closest_point(IntPoint::new(10, 10)), IntPoint::new(10, 0));
        assert_eq!(
            s.closest_point(IntPoint::new(10, -10)),
            IntPoint::new(10, 0)
        );
    }

    #[test]
    fn test_4() {
        let s = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        assert_eq!(s.closest_point(IntPoint::new(-10, 0)), IntPoint::new(0, 0));
    }

    #[test]
    fn test_5() {
        let s = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        assert_eq!(s.closest_point(IntPoint::new(20, 0)), IntPoint::new(10, 0));
    }
}