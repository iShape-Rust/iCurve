use core::ptr;
use crate::int::math::point::IntPoint;
use crate::int::math::x_segment::XSegment;

pub(crate) struct FourConvexHull {
    point_len: usize,
    segment_len: usize,
    points: [IntPoint; 4],
    segments: [XSegment; 4]
}

impl FourConvexHull {
    #[inline]
    pub(crate) fn set(&mut self, source: &[IntPoint]) {
        let len = source.len();
        debug_assert!(len > 1);
        debug_assert!(len <= 4);
        unsafe {
            ptr::copy_nonoverlapping(source.as_ptr(), self.points.as_mut_ptr(), len);
        }
        self.point_len = len;

        if len == 2 {
            self.segment_len = 1;
            self.segments[0] = if source[0] < source[1] {
                XSegment { a: source[0], b: source[1] }
            } else {
                XSegment { a: source[1], b: source[0] }
            };
        } else {
            self.segment_len = len;
        }
    }

    #[inline]
    pub(crate) fn is_overlap(&self, other: &Self) -> bool {
        for s0 in self.segments[0..self.segment_len].iter() {
            for s1 in self.segments[0..other.segment_len].iter() {
                if s0.is_overlap(s1) {
                   return true
                }
            }
        }
        // possible that one inside other

        if self.contains(*other.points.first().unwrap()) {
            return true;
        }

        if other.contains(*self.points.first().unwrap()) {
            return true;
        }

        false
    }
}

impl Default for FourConvexHull {
    #[inline]
    fn default() -> Self {
        Self {
            point_len: 0,
            segment_len: 0,
            points: [IntPoint::zero(); 4],
            segments: [XSegment { a: IntPoint::zero(), b: IntPoint::zero() }; 4],
        }
    }
}

pub trait Convex {
    fn contains(&self, point: IntPoint) -> bool;
}

impl Convex for FourConvexHull {

    #[inline]
    fn contains(&self, p: IntPoint) -> bool {
        if self.point_len <= 2 {
            return false;
        }

        let mut a = self.points[self.point_len - 1];
        for &b in self.points[0..self.point_len].iter() {
            let v0 = p - a;
            let v1 = b - a;

            let cross = v0.cross_product(&v1);
            if cross > 0 {
                return false
            }
            a = b;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::int::convex::hull::{Convex, FourConvexHull};
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_0() {
        let mut convex_hull = FourConvexHull::default();

        convex_hull.set(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]);

        assert!(convex_hull.contains(IntPoint::new(1, 1)));
        assert!(convex_hull.contains(IntPoint::new(1, 9)));
        assert!(convex_hull.contains(IntPoint::new(9, 9)));
        assert!(convex_hull.contains(IntPoint::new(9, 1)));

        assert!(!convex_hull.contains(IntPoint::new(-1, -1)));
        assert!(!convex_hull.contains(IntPoint::new(-1, 1)));
        assert!(!convex_hull.contains(IntPoint::new(-1, 9)));
        assert!(!convex_hull.contains(IntPoint::new(-1, 11)));

        assert!(!convex_hull.contains(IntPoint::new(11, -1)));
        assert!(!convex_hull.contains(IntPoint::new(11, 1)));
        assert!(!convex_hull.contains(IntPoint::new(11, 9)));
        assert!(!convex_hull.contains(IntPoint::new(11, 11)));

        assert!(!convex_hull.contains(IntPoint::new(-1, -1)));
        assert!(!convex_hull.contains(IntPoint::new(1, -1)));
        assert!(!convex_hull.contains(IntPoint::new(9, -1)));
        assert!(!convex_hull.contains(IntPoint::new(11, -1)));

        assert!(!convex_hull.contains(IntPoint::new(-1, 11)));
        assert!(!convex_hull.contains(IntPoint::new(1, 11)));
        assert!(!convex_hull.contains(IntPoint::new(9, 11)));
        assert!(!convex_hull.contains(IntPoint::new(11, 11)));

        assert!(convex_hull.contains(IntPoint::new(0, 0)));
        assert!(convex_hull.contains(IntPoint::new(5, 0)));
        assert!(convex_hull.contains(IntPoint::new(10, 0)));

        assert!(convex_hull.contains(IntPoint::new(0, 10)));
        assert!(convex_hull.contains(IntPoint::new(5, 10)));
        assert!(convex_hull.contains(IntPoint::new(10, 10)));

        assert!(convex_hull.contains(IntPoint::new(0, 5)));
        assert!(convex_hull.contains(IntPoint::new(10, 5)));
    }
}