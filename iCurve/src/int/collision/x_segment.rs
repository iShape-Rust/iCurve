use crate::int::collision::colliding::{Colliding, CollidingResult};
use crate::int::math::ab_segment::{Contain, IntABSegment};
use crate::int::math::range::LineRange;
use crate::int::math::triangle::Triangle;
use crate::int::math::x_segment::XSegment;

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

    #[inline]
    fn overlap(&self, other: &Self) -> bool {
        self.collide(other) != CollidingResult::None
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
    pub(crate) fn is_overlap_y(&self, other: &Self) -> bool {
        self.y_range().is_overlap(&other.y_range())
    }
    
    #[inline(always)]
    pub(crate) fn is_overlap_xy(&self, other: &Self) -> bool {
        if self.x_range().is_not_overlap(&other.x_range()) {
            return false;
        }
        self.is_overlap_y(other)
    }

    #[inline]
    fn sqr_len(&self) -> u64 {
        self.a.sqr_dist(&self.b)
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
        match self.contains_on_collinear_span(other.a) {
            Contain::Inside => return CollidingResult::Overlap,
            Contain::End => ends += 1,
            Contain::Outside => {}
        }

        match self.contains_on_collinear_span(other.b) {
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
    fn test_15() {
        let s0 = XSegment::new(IntPoint::new(-200, 351), IntPoint::new(167, 141));
        let s1 = XSegment::new(IntPoint::new(150, 150), IntPoint::new(200, 200));

        assert_eq!(s0.collide(&s1), CollidingResult::None);
    }
}
