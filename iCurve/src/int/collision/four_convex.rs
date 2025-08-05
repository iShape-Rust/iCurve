use crate::data::four_vec::FourVec;
use crate::int::collision::colliding::{Colliding, CollidingResult};
use crate::int::collision::convexity::Convexity;
use crate::int::math::point::IntPoint;
use crate::int::math::x_segment::XSegment;

struct FourConvex {
    len: usize,
    points: [IntPoint; 4],
    segments: [XSegment; 4],
}

impl FourConvex {
    #[inline]
    fn new(offset: IntPoint, source: &[IntPoint]) -> Self {
        let len = source.len();
        debug_assert!(len > 1);
        debug_assert!(len <= 4);

        let mut points = [IntPoint::zero(); 4];
        for i in 0..len {
            points[i] = source[i] - offset
        }

        let mut segments = [XSegment::default(); 4];
        let mut a = source[len - 1];
        for (i, &b) in source.iter().enumerate() {
            segments[i] = XSegment::new(a, b);
            a = b;
        }

        Self {
            len,
            points,
            segments,
        }
    }

    #[inline]
    fn overlap(&self, other: &Self) -> CollidingResult {
        let mut touch = false;
        for s0 in self.segments[0..self.len].iter() {
            for s1 in other.segments[0..other.len].iter() {
                match s0.collide(s1) {
                    CollidingResult::Overlap => return CollidingResult::Overlap,
                    CollidingResult::Touch => touch = true,
                    CollidingResult::None => {}
                }
            }
        }

        if touch {
            return CollidingResult::Touch;
        }

        if self.points.convex_contains(other.points[0]) {
            return CollidingResult::Overlap;
        }

        if other.points.convex_contains(self.points[0]) {
            return CollidingResult::Overlap;
        }

        CollidingResult::None
    }
}

impl FourVec<IntPoint> {

    #[inline]
    pub(crate) fn overlap_with_segment(&self, s: &XSegment) -> bool {
        let points = self.slice();
        let mut p0 = *points.last().unwrap();

        for &pi in points.iter() {
            let si = XSegment::new(p0, pi);
            match si.collide(s) {
                CollidingResult::Overlap | CollidingResult::Touch => return true,
                _ => {}
            }

            p0 = pi;
        }

        false
    }
}
