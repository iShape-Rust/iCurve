use crate::int::math::point::IntPoint;

pub trait Convexity {
    fn is_convex(&self) -> bool;
    fn convex_contains(&self, point: IntPoint) -> bool;
    fn overlaps_with_unit_margin(&self, other: &Self) -> bool;
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

    #[inline]
    fn overlaps_with_unit_margin(&self, other: &Self) -> bool {
        if find_separate_unit_line(self, other) {
            return false;
        }

        let separation = find_separate_unit_line(other, self);
        
        !separation
    }
}

fn find_separate_unit_line(path: &[IntPoint], points: &[IntPoint]) -> bool {
    const MIN_AREA: u64 = i32::MAX as u64;

    let mut a = *path.last().unwrap();

    'main_loop: for &b in path.iter() {
        let ba = b - a;
        let mut min_s = u64::MAX;
        let mut min_ba = IntPoint::zero();
        for &p in points.iter() {
            let ap = a - p;
            let s = ba.cross_product(&ap);
            if s <= 0 {
                a = b;
                continue 'main_loop;
            }
            if min_s > s as u64 {
                min_s = s as u64;
                min_ba = ba;
            }
        }

        // all points from one side
        if min_s >= MIN_AREA {
            // the nearest point already is too far
            return true;
        }

        let sqr_s = min_s * min_s;
        let sqr_ba = min_ba.sqr_len();

        if sqr_s > sqr_ba {
            // the nearest point is more than 1 unit far away
            return true;
        }

        a = b;
    }

    false
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

    #[test]
    fn test_overlaps_with_unit_margin_0() {
        let path_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let path_1 = [
            IntPoint::new(10, 5),
            IntPoint::new(20, 0),
            IntPoint::new(20, 10),
        ];

        let overlap_0 = path_0.overlaps_with_unit_margin(&path_1);
        let overlap_1 = path_1.overlaps_with_unit_margin(&path_0);

        assert!(overlap_0);
        assert!(overlap_1);
    }

    #[test]
    fn test_overlaps_with_unit_margin_1() {
        let path_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let path_1 = [
            IntPoint::new(11, 5),
            IntPoint::new(20, 0),
            IntPoint::new(20, 10),
        ];

        let overlap_0 = path_0.overlaps_with_unit_margin(&path_1);
        let overlap_1 = path_1.overlaps_with_unit_margin(&path_0);

        assert!(overlap_0);
        assert!(overlap_1);
    }

    #[test]
    fn test_overlaps_with_unit_margin_2() {
        let path_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let path_1 = [
            IntPoint::new(12, 5),
            IntPoint::new(20, 0),
            IntPoint::new(20, 10),
        ];

        let overlap_0 = path_0.overlaps_with_unit_margin(&path_1);
        let overlap_1 = path_1.overlaps_with_unit_margin(&path_0);

        assert!(!overlap_0);
        assert!(!overlap_1);
    }

    #[test]
    fn test_overlaps_with_unit_margin_3() {
        let path_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let path_1 = [
            IntPoint::new(10, 5),
            IntPoint::new(20, 0)
        ];

        let overlap_0 = path_0.overlaps_with_unit_margin(&path_1);
        let overlap_1 = path_1.overlaps_with_unit_margin(&path_0);

        assert!(overlap_0);
        assert!(overlap_1);
    }

    #[test]
    fn test_overlaps_with_unit_margin_4() {
        let path_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let path_1 = [
            IntPoint::new(11, 5),
            IntPoint::new(20, 0)
        ];

        let overlap_0 = path_0.overlaps_with_unit_margin(&path_1);
        let overlap_1 = path_1.overlaps_with_unit_margin(&path_0);

        assert!(overlap_0);
        assert!(overlap_1);
    }

    #[test]
    fn test_overlaps_with_unit_margin_5() {
        let path_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let path_1 = [
            IntPoint::new(12, 5),
            IntPoint::new(20, 0)
        ];

        let overlap_0 = path_0.overlaps_with_unit_margin(&path_1);
        let overlap_1 = path_1.overlaps_with_unit_margin(&path_0);

        assert!(!overlap_0);
        assert!(!overlap_1);
    }

    #[test]
    fn test_overlaps_with_unit_margin_6() {
        let path_0 = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ];

        let path_1 = [
            IntPoint::new(5, 20),
            IntPoint::new(20, 5)
        ];

        let overlap_0 = path_0.overlaps_with_unit_margin(&path_1);
        let overlap_1 = path_1.overlaps_with_unit_margin(&path_0);

        assert!(!overlap_0);
        assert!(!overlap_1);
    }
}
