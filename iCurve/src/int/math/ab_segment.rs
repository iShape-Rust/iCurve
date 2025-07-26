use crate::int::math::point::IntPoint;
use crate::int::math::segment::IntSegment;
use crate::int::math::x_segment::XSegment;
use core::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub(crate) enum Contain {
    Inside,
    Outside,
    End,
}

pub(crate) trait IntABSegment {
    fn a(&self) -> IntPoint;
    fn b(&self) -> IntPoint;

    #[inline]
    fn is_on_collinear_span(&self, p: IntPoint) -> bool {
        let v0 = p - self.a();
        let v1 = p - self.b();
        let dot = v0.dot_product(&v1);

        dot <= 0
    }

    #[inline]
    fn contains_on_collinear_span(&self, p: IntPoint) -> Contain {
        let v0 = p - self.a();
        let v1 = p - self.b();
        let dot = v0.dot_product(&v1);

        match dot.cmp(&0) {
            Ordering::Less => Contain::Inside,
            Ordering::Equal => Contain::End,
            Ordering::Greater => Contain::Outside,
        }
    }

    #[inline]
    fn is_on_span(&self, p: IntPoint) -> bool {
        let v0 = p - self.a();
        let v1 = p - self.b();

        if v0.cross_product(&v1) != 0 {
            return false;
        }

        let dot = v0.dot_product(&v1);

        dot <= 0
    }

    #[inline]
    fn contains(&self, p: IntPoint) -> Contain {
        let v0 = p - self.a();
        let v1 = p - self.b();

        if v0.cross_product(&v1) != 0 {
            return Contain::Outside;
        }

        let dot = v0.dot_product(&v1);

        match dot.cmp(&0) {
            Ordering::Less => Contain::Inside,
            Ordering::Equal => Contain::End,
            Ordering::Greater => Contain::Outside,
        }
    }

    #[inline]
    fn is_collinear(&self, other: &Self) -> bool {
        let v0 = self.a() - self.b();
        let v1 = other.a() - other.b();
        
        v0.cross_product(&v1) == 0
    }
}

impl IntABSegment for XSegment {
    #[inline(always)]
    fn a(&self) -> IntPoint {
        self.a
    }

    #[inline(always)]
    fn b(&self) -> IntPoint {
        self.b
    }
}

impl IntABSegment for IntSegment {
    #[inline(always)]
    fn a(&self) -> IntPoint {
        self.a
    }

    #[inline(always)]
    fn b(&self) -> IntPoint {
        self.b
    }
}
#[cfg(test)]
mod tests {
    use crate::int::math::ab_segment::{Contain, IntABSegment};
    use crate::int::math::point::IntPoint;
    use crate::int::math::segment::IntSegment;

    #[test]
    fn test_contains_0() {
        let result = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0))
            .contains(IntPoint::new(5, 0));
        assert_eq!(result, Contain::Inside);
    }

    #[test]
    fn test_contains_1() {
        let result = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0))
            .contains(IntPoint::new(-5, 0));
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_contains_2() {
        let result = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0))
            .contains(IntPoint::new(15, 0));
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_contains_3() {
        let result = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0))
            .contains(IntPoint::new(0, 0));
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_contains_4() {
        let result = IntSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0))
            .contains(IntPoint::new(10, 0));
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_contains_5() {
        let result = IntSegment::new(IntPoint::new(5, 5), IntPoint::new(10, 10))
            .contains(IntPoint::new(15, 15));
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_contains_6() {
        let result = IntSegment::new(IntPoint::new(5, 5), IntPoint::new(10, 10))
            .contains(IntPoint::new(0, 0));
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_contains_7() {
        let result = IntSegment::new(IntPoint::new(5, 5), IntPoint::new(10, 10))
            .contains(IntPoint::new(5, 5));
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_contains_8() {
        let result = IntSegment::new(IntPoint::new(5, 5), IntPoint::new(10, 10))
            .contains(IntPoint::new(10, 10));
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_contains_9() {
        let result = IntSegment::new(IntPoint::new(5, 5), IntPoint::new(10, 10))
            .contains(IntPoint::new(7, 7));
        assert_eq!(result, Contain::Inside);
    }
}
