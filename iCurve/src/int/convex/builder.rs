use alloc::vec::Vec;
use core::cmp::Ordering;
use crate::int::math::point::IntPoint;

pub struct FourConvexBuilder {
    buffer: Vec<IntPoint>,
    result: Vec<IntPoint>,
}

impl FourConvexBuilder {
    pub fn build(&mut self, points: &[IntPoint]) -> &[IntPoint] {
        debug_assert!(!points.is_empty());
        debug_assert!(points.len() <= 4);
        self.result.clear();
        self.buffer.clear();
        self.buffer.extend_from_slice(points);

        let a = self.buffer.left_most();
        self.result.push(a);

        let b = if self.buffer.is_empty() {
            return &self.result;
        } else {
            self.buffer.edge::<true>(a)
        };

        self.result.push(b);

        let c = if self.buffer.is_empty() {
            return &self.result;
        } else {
            self.buffer.edge::<false>(a)
        };

        // only one last point is possible
        if let Some(&d) = self.buffer.first() {
            let bc = b - c;
            let bd = b - d;
            let cross = bd.cross_product(&bc);
            if cross > 0 {
                self.result.push(d);
            }
        }

        self.result.push(c);

        &self.result
    }
}

impl Default for FourConvexBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(4),
            result: Vec::with_capacity(4),
        }
    }
}

trait Util {
    fn left_most(&mut self) -> IntPoint;
    fn edge<const REVERSED: bool>(&mut self, a: IntPoint) -> IntPoint;
}

impl Util for Vec<IntPoint> {
    #[inline]
    fn left_most(&mut self) -> IntPoint {
        let mut a = self[0];
        let mut j = 0;
        let mut i = 1;
        while i < self.len() {
            let p = self[i];
            match p.cmp(&a) {
                Ordering::Less => {
                    j = i;
                    a = p;
                    i += 1;
                }
                Ordering::Greater => i += 1,
                Ordering::Equal => _ = self.swap_remove(i),
            }
        }

        self.swap_remove(j)
    }

    #[inline]
    fn edge<const REVERSED: bool>(&mut self, a: IntPoint) -> IntPoint {
        debug_assert!(!self.is_empty());

        let mut j = 0;
        let mut i = 1;
        let mut b = self[0];
        let mut e = b - a;

        while i < self.len() {
            let p = self[i];
            let v = p - a;
            let cross = v.cross_product(&e);

            let ord = if REVERSED {
                0.cmp(&cross)
            } else {
                cross.cmp(&0)
            };

            match ord {
                Ordering::Less => {
                    j = i;
                    b = p;
                    e = v;
                    i += 1;
                }
                Ordering::Greater => i += 1,
                Ordering::Equal => {
                    self.swap_remove(i);
                    if a.sqr_len(&b) < a.sqr_len(&p) {
                        self[j] = p;
                        b = p;
                        e = v;
                    }
                }
            }
        }
        self.swap_remove(j)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;
    use rand::Rng;
    use crate::int::convex::builder::FourConvexBuilder;
    use crate::int::math::point::IntPoint;

    trait ConvexTest {
        fn is_convex(&self) -> bool;
    }

    impl ConvexTest for [IntPoint] {
        fn is_convex(&self) -> bool {
            let n = self.len();
            if n <= 2 {
                return true;
            }

            let a = self[n - 2];
            let mut b = self[n - 1];
            let mut ab = a - b;
            for &c in self.iter() {
                let bc = b - c;
                if bc.cross_product(&ab) > 0 {
                    return false;
                }
                b = c;
                ab = bc;
            }

            true
        }
    }

    #[test]
    fn test_0() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![IntPoint::new(0, 0)]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert!(result.is_convex());
    }

    #[test]
    fn test_1() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![IntPoint::new(0, 0), IntPoint::new(0, 0)]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert!(result.is_convex());
    }

    #[test]
    fn test_2() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![IntPoint::new(0, 0), IntPoint::new(10, 0)]);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, 0));
        assert!(result.is_convex());
    }

    #[test]
    fn test_3() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![IntPoint::new(10, 0), IntPoint::new(0, 0)]);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, 0));
        assert!(result.is_convex());
    }

    #[test]
    fn test_4() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, -10),
            IntPoint::new(10, 10),
        ]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, -10));
        assert_eq!(result[2], IntPoint::new(10, 10));
        assert!(result.is_convex());
    }

    #[test]
    fn test_5() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, -10));
        assert_eq!(result[2], IntPoint::new(10, 10));
        assert!(result.is_convex());
    }

    #[test]
    fn test_6() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(20, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, -10));
        assert_eq!(result[2], IntPoint::new(20, 0));
        assert_eq!(result[3], IntPoint::new(10, 10));
        assert!(result.is_convex());
    }

    #[test]
    fn test_7() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, -10));
        assert_eq!(result[2], IntPoint::new(10, 10));
        assert!(result.is_convex());
    }

    #[test]
    fn test_8() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, -10));
        assert_eq!(result[2], IntPoint::new(10, 10));
        assert!(result.is_convex());
    }

    #[test]
    fn test_9() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 0),
        ]);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(15, 0));
        assert!(result.is_convex());
    }

    #[test]
    fn test_10() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 10),
        ]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, 0));
        assert_eq!(result[2], IntPoint::new(15, 10));
        assert!(result.is_convex());
    }

    #[test]
    fn test_11() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(0, 5),
            IntPoint::new(0, 10),
            IntPoint::new(0, 15),
        ]);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(0, 15));
        assert!(result.is_convex());
    }

    #[test]
    fn test_12() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(0, -5),
            IntPoint::new(0, -10),
            IntPoint::new(0, -15),
        ]);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], IntPoint::new(0, -15));
        assert_eq!(result[1], IntPoint::new(0, 0));
        assert!(result.is_convex());
    }

    #[test]
    fn test_13() {
        let mut builder = FourConvexBuilder::default();
        let result = builder.build(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(10, -10),
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
        ]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], IntPoint::new(0, 0));
        assert_eq!(result[1], IntPoint::new(10, -10));
        assert_eq!(result[2], IntPoint::new(10, 10));
        assert!(result.is_convex());
    }

    #[test]
    fn test_random() {
        let mut builder = FourConvexBuilder::default();
        let mut points = Vec::with_capacity(4);
        let range = -1000i64..=1000i64;
        let mut rng = rand::rng();
        for _ in 0..1000 {
            points.clear();
            let x = rng.random_range(range.clone());
            let y = rng.random_range(range.clone());
            points.push(IntPoint::new(x, y));

            let result = builder.build(&points);
            assert!(result.is_convex());
        }
    }
}