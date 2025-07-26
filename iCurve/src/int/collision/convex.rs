use crate::data::four_vec::FourVec;
use crate::int::collision::colliding::{Colliding, CollidingResult};
use crate::int::collision::convexity::Convexity;
use crate::int::math::point::IntPoint;
use crate::int::math::x_segment::XSegment;

impl Colliding for FourVec<IntPoint> {
    #[inline]
    fn collide(&self, other: &Self) -> CollidingResult {
        let offset = self.buffer[0];
        FourConvex::new(offset, self.slice()).collide(&FourConvex::new(offset, other.slice()))
    }

    #[inline]
    fn overlap(&self, other: &Self) -> bool {
        self.collide(other) != CollidingResult::None
    }
}

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
}

impl Colliding for FourConvex {
    #[inline]
    fn collide(&self, other: &Self) -> CollidingResult {
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

    #[inline]
    fn overlap(&self, other: &Self) -> bool {
        self.collide(other) != CollidingResult::None
    }
}

#[cfg(test)]
mod tests {
    use crate::int::collision::colliding::{Colliding, CollidingResult};
    use crate::int::collision::convex_hull::FourConvexPathExt;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_0() {
        let hull_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]
        .to_convex_hull();

        let hull_1 = [
            IntPoint::new(8, 5),
            IntPoint::new(13, 0),
            IntPoint::new(18, 5),
            IntPoint::new(13, 10),
        ]
        .to_convex_hull();

        assert_eq!(hull_0.collide(&hull_1), CollidingResult::Overlap);
    }

    #[test]
    fn test_1() {
        let hull_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 5),
            IntPoint::new(0, 10),
        ]
        .to_convex_hull();

        let hull_1 = [
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
            IntPoint::new(10, 0),
        ]
        .to_convex_hull();

        assert_eq!(hull_0.collide(&hull_1), CollidingResult::Touch);
    }

    #[test]
    fn test_2() {
        let hull_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 5),
            IntPoint::new(0, 10),
        ]
            .to_convex_hull();

        let hull_1 = [
            IntPoint::new(4, 5),
            IntPoint::new(9, 10),
            IntPoint::new(9, 0),
        ]
            .to_convex_hull();

        assert_eq!(hull_0.collide(&hull_1), CollidingResult::Overlap);
    }

    #[test]
    fn test_3() {
        let hull_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 5),
            IntPoint::new(0, 10),
        ]
            .to_convex_hull();

        let hull_1 = [
            IntPoint::new(10, 5),
            IntPoint::new(15, 10),
            IntPoint::new(15, 0),
        ]
            .to_convex_hull();

        assert_eq!(hull_0.collide(&hull_1), CollidingResult::None);
    }

    #[test]
    fn test_4() {
        let hull_0 = [
            IntPoint::new(0, -100),
            IntPoint::new(0, 100),
            IntPoint::new(100, 0),
            IntPoint::new(-100, 0),
        ]
            .to_convex_hull();

        let hull_1 = [
            IntPoint::new(100, 100),
            IntPoint::new(100, 200),
            IntPoint::new(200, 100),
            IntPoint::new(200, 200),
        ]
            .to_convex_hull();

        assert_eq!(hull_0.collide(&hull_1), CollidingResult::None);
    }
}
