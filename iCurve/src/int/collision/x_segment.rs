use crate::int::collision::colliding::{Colliding, CollidingResult};
use crate::int::math::point::IntPoint;
use crate::int::math::range::LineRange;
use crate::int::math::triangle::Triangle;
use crate::int::math::x_segment::XSegment;
use core::cmp::Ordering;

impl Colliding for XSegment {
    #[inline]
    fn collide(&self, other: &Self) -> CollidingResult {
        if !self.is_overlap_xy(other) {
            return CollidingResult::None;
        }

        let a0b0a1 = Triangle::clock_direction(self.a, self.b, other.a);
        let a0b0b1 = Triangle::clock_direction(self.a, self.b, other.b);

        let a1b1a0 = Triangle::clock_direction(other.a, other.b, self.a);
        let a1b1b0 = Triangle::clock_direction(other.a, other.b, self.b);

        let no_degenerate = a0b0a1 != 0 && a0b0b1 != 0 && a1b1a0 != 0 && a1b1b0 != 0;
        if no_degenerate {
            return if a0b0a1 != a0b0b1 && a1b1a0 != a1b1b0 {
                CollidingResult::Overlap
            } else {
                CollidingResult::None
            };
        }

        let collinear = (self.a - self.b).cross_product(&(other.a - other.b)) == 0;

        if collinear {
            self.collinear_collide(other)
        } else {
            self.not_collinear_collide(other)            
        }
    }
}

#[derive(Debug, PartialEq)]
enum Contain {
    Inside,
    Outside,
    End,
}

#[inline]
fn ab_contains(a: IntPoint, b: IntPoint, p: IntPoint) -> Contain {
    let v0 = p - a;
    let v1 = p - b;

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
fn ab_include(a: IntPoint, b: IntPoint, p: IntPoint) -> Contain {
    let v0 = p - a;
    let v1 = p - b;
    let dot = v0.dot_product(&v1);

    match dot.cmp(&0) {
        Ordering::Less => Contain::Inside,
        Ordering::Equal => Contain::End,
        Ordering::Greater => Contain::Outside,
    }
}

impl XSegment {
    #[inline(always)]
    fn x_range(&self) -> LineRange {
        LineRange {
            min: self.a.x,
            max: self.b.x,
        }
    }

    #[inline(always)]
    fn y_range(&self) -> LineRange {
        LineRange::new(self.a.y, self.b.y)
    }

    #[inline(always)]
    fn is_overlap_xy(&self, other: &Self) -> bool {
        if self.x_range().is_not_overlap(&other.x_range()) {
            return false;
        }
        self.y_range().is_overlap(&other.y_range())
    }

    #[inline]
    fn contains(&self, p: IntPoint) -> Contain {
        ab_contains(self.a, self.b, p)
    }

    #[inline]
    fn sqr_len(&self) -> u64 {
        self.a.sqr_len(&self.b)
    }

    #[inline]
    fn include(&self, p: IntPoint) -> Contain {
        ab_include(self.a, self.b, p)
    }

    #[inline]
    fn not_collinear_collide(&self, other: &Self) -> CollidingResult {
        match self.contains(other.a) {
            Contain::Inside => return CollidingResult::Overlap,
            Contain::End => return CollidingResult::Touch,
            Contain::Outside => {}
        }

        match self.contains(other.b) {
            Contain::Inside => return CollidingResult::Overlap,
            Contain::End => return CollidingResult::Touch,
            Contain::Outside => {}
        }

        CollidingResult::None
    }

    #[inline]
    fn collinear_collide(&self, other: &Self) -> CollidingResult {
        if self.sqr_len() >= other.sqr_len() {
            self.ordered_collinear_collide(other)
        } else {
            other.ordered_collinear_collide(self)
        }
    }

    #[inline]
    fn ordered_collinear_collide(&self, other: &Self) -> CollidingResult {
        let mut ends = 0;
        match self.include(other.a) {
            Contain::Inside => return CollidingResult::Overlap,
            Contain::End => ends += 1,
            Contain::Outside => {}
        }

        match self.include(other.b) {
            Contain::Inside => return CollidingResult::Overlap,
            Contain::End => ends += 1,
            Contain::Outside => {}
        }

        match ends {
            0 => CollidingResult::None,
            1 => CollidingResult::Touch,
            _ => CollidingResult::Overlap
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::int::collision::colliding::{Colliding, CollidingResult};
    use crate::int::collision::x_segment::{Contain, ab_contains};
    use crate::int::math::point::IntPoint;
    use crate::int::math::x_segment::XSegment;

    #[test]
    fn test_0() {
        let s0 = XSegment::new(IntPoint::new(0, -5), IntPoint::new(0, 5));
        let s1 = XSegment::new(IntPoint::new(-5, 0), IntPoint::new(5, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::Overlap);
    }

    #[test]
    fn test_1() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(5, 0), IntPoint::new(5, 10));

        assert_eq!(s0.collide(&s1), CollidingResult::Overlap);
    }

    #[test]
    fn test_2() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(0, 10));

        assert_eq!(s0.collide(&s1), CollidingResult::Touch);
    }

    #[test]
    fn test_3() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(0, 10));
        let s1 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::Touch);
    }

    #[test]
    fn test_4() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(10, 0), IntPoint::new(10, -10));

        assert_eq!(s0.collide(&s1), CollidingResult::Touch);
    }

    #[test]
    fn test_5() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(0, -10));

        assert_eq!(s0.collide(&s1), CollidingResult::Touch);
    }

    #[test]
    fn test_6() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(10, 0), IntPoint::new(20, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::Touch);
    }

    #[test]
    fn test_7() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(15, 0), IntPoint::new(20, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::None);
    }

    #[test]
    fn test_8() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(15, 0), IntPoint::new(5, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::Overlap);
    }

    #[test]
    fn test_9() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::Overlap);
    }

    #[test]
    fn test_10() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(15, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::Overlap);
    }

    #[test]
    fn test_11() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(5, 0), IntPoint::new(15, 0));

        assert_eq!(s0.collide(&s1), CollidingResult::Overlap);
    }

    #[test]
    fn test_12() {
        let s0 = XSegment::new(IntPoint::new(-10, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(-25, -10), IntPoint::new(0, 10));

        assert_eq!(s0.collide(&s1), CollidingResult::None);
    }

    #[test]
    fn test_13() {
        let s0 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(0, 0), IntPoint::new(-10, 10));

        assert_eq!(s0.collide(&s1), CollidingResult::Touch);
    }

    #[test]
    fn test_14() {
        let s0 = XSegment::new(IntPoint::new(0, 10), IntPoint::new(10, 0));
        let s1 = XSegment::new(IntPoint::new(10, 10), IntPoint::new(10, 20));

        assert_eq!(s0.collide(&s1), CollidingResult::None);
    }

    #[test]
    fn test_ab_contains_0() {
        let result = ab_contains(
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(5, 0),
        );
        assert_eq!(result, Contain::Inside);
    }

    #[test]
    fn test_ab_contains_1() {
        let result = ab_contains(
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(-5, 0),
        );
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_ab_contains_2() {
        let result = ab_contains(
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 0),
        );
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_ab_contains_3() {
        let result = ab_contains(
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(0, 0),
        );
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_ab_contains_4() {
        let result = ab_contains(
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 0),
        );
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_ab_contains_5() {
        let result = ab_contains(
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
            IntPoint::new(15, 15),
        );
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_ab_contains_6() {
        let result = ab_contains(
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
            IntPoint::new(0, 0),
        );
        assert_eq!(result, Contain::Outside);
    }

    #[test]
    fn test_ab_contains_7() {
        let result = ab_contains(
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
            IntPoint::new(5, 5),
        );
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_ab_contains_8() {
        let result = ab_contains(
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
            IntPoint::new(10, 10),
        );
        assert_eq!(result, Contain::End);
    }

    #[test]
    fn test_ab_contains_9() {
        let result = ab_contains(
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
            IntPoint::new(7, 7),
        );
        assert_eq!(result, Contain::Inside);
    }
}
