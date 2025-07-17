use core::cmp::Ordering;
use crate::int::math::point::IntPoint;
use crate::int::math::range::LineRange;
use crate::int::math::triangle::Triangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct XSegment {
    pub(crate) a: IntPoint,
    pub(crate) b: IntPoint,
}

impl PartialOrd for XSegment {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for XSegment {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.a.cmp(&other.a);
        if a == Ordering::Equal {
            self.b.cmp(&other.b)
        } else {
            a
        }
    }
}

impl XSegment {

    #[inline(always)]
    fn x_range(&self) -> LineRange {
        LineRange { min: self.a.x, max: self.b.x }
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
    
    #[inline(always)]
    pub(crate) fn is_overlap(&self, other: &Self) -> bool {
        if !self.is_overlap_xy(other) {
            return false;
        }

        let a0b0a1 = Triangle::clock_direction(self.a, self.b, other.a);
        let a0b0b1 = Triangle::clock_direction(self.a, self.b, other.b);

        let a1b1a0 = Triangle::clock_direction(other.a, other.b, self.a);
        let a1b1b0 = Triangle::clock_direction(other.a, other.b, self.b);

        a0b0a1 != a0b0b1 && a1b1a0 != a1b1b0
    }
}