use crate::int::math::point::IntPoint;

pub trait Convexity {
    fn is_convex(&self) -> bool;
    fn convex_contains(&self, point: IntPoint) -> bool;
}

impl Convexity for [IntPoint] {
    #[inline]
    fn is_convex(&self) -> bool {
        let n = self.len();
        if n <= 2 {
            return true;
        }

        let p0 = self[n - 2];
        let p1 = self[n - 1];
        let mut pi = self[0];

        let e0 = p1 - p0;
        let mut ei = pi - p1;
        let sign = e0.cross_product(&ei) > 0;

        for &p in self.iter().skip(1) {
            let e = p - pi;
            if (ei.cross_product(&e) > 0) != sign {
                return false;
            }
            pi = p;
            ei = e;
        }

        true
    }

    #[inline]
    fn convex_contains(&self, p: IntPoint) -> bool {
        if self.len() <= 2 {
            return false;
        }

        let mut a = self[self.len() - 1];
        for &b in self.iter() {
            let v0 = p - a;
            let v1 = b - a;

            let cross = v0.cross_product(&v1);
            if cross > 0 {
                return false;
            }
            a = b;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::int::collision::convexity::Convexity;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_0() {
        let convex = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]
        .is_convex();

        assert!(convex);
    }

    #[test]
    fn test_1() {
        let convex = [
            IntPoint::new(0, 0),
            IntPoint::new(0, 10),
            IntPoint::new(10, 10),
            IntPoint::new(10, 0),
        ]
        .is_convex();

        assert!(convex);
    }

    #[test]
    fn test_2() {
        let convex = [
            IntPoint::new(0, 0),
            IntPoint::new(0, 10),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
        ]
        .is_convex();

        assert!(!convex);
    }

    #[test]
    fn test_3() {
        let convex_hull = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        assert!(convex_hull.convex_contains(IntPoint::new(1, 1)));
        assert!(convex_hull.convex_contains(IntPoint::new(1, 9)));
        assert!(convex_hull.convex_contains(IntPoint::new(9, 9)));
        assert!(convex_hull.convex_contains(IntPoint::new(9, 1)));

        assert!(!convex_hull.convex_contains(IntPoint::new(-1, -1)));
        assert!(!convex_hull.convex_contains(IntPoint::new(-1, 1)));
        assert!(!convex_hull.convex_contains(IntPoint::new(-1, 9)));
        assert!(!convex_hull.convex_contains(IntPoint::new(-1, 11)));

        assert!(!convex_hull.convex_contains(IntPoint::new(11, -1)));
        assert!(!convex_hull.convex_contains(IntPoint::new(11, 1)));
        assert!(!convex_hull.convex_contains(IntPoint::new(11, 9)));
        assert!(!convex_hull.convex_contains(IntPoint::new(11, 11)));

        assert!(!convex_hull.convex_contains(IntPoint::new(-1, -1)));
        assert!(!convex_hull.convex_contains(IntPoint::new(1, -1)));
        assert!(!convex_hull.convex_contains(IntPoint::new(9, -1)));
        assert!(!convex_hull.convex_contains(IntPoint::new(11, -1)));

        assert!(!convex_hull.convex_contains(IntPoint::new(-1, 11)));
        assert!(!convex_hull.convex_contains(IntPoint::new(1, 11)));
        assert!(!convex_hull.convex_contains(IntPoint::new(9, 11)));
        assert!(!convex_hull.convex_contains(IntPoint::new(11, 11)));

        assert!(convex_hull.convex_contains(IntPoint::new(0, 0)));
        assert!(convex_hull.convex_contains(IntPoint::new(5, 0)));
        assert!(convex_hull.convex_contains(IntPoint::new(10, 0)));

        assert!(convex_hull.convex_contains(IntPoint::new(0, 10)));
        assert!(convex_hull.convex_contains(IntPoint::new(5, 10)));
        assert!(convex_hull.convex_contains(IntPoint::new(10, 10)));

        assert!(convex_hull.convex_contains(IntPoint::new(0, 5)));
        assert!(convex_hull.convex_contains(IntPoint::new(10, 5)));
    }
}
