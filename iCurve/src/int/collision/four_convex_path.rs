use crate::data::four_vec::FourVec;
use crate::int::math::point::IntPoint;
use core::cmp::Ordering;

pub trait FourConvexPath {
    fn to_four_convex(&self) -> FourVec<IntPoint>;
}

impl FourConvexPath for [IntPoint] {
    #[inline]
    fn to_four_convex(&self) -> FourVec<IntPoint> {
        debug_assert!(!self.is_empty());
        debug_assert!(self.len() <= 4);
        let mut result = FourVec::new();
        let mut buffer = FourVec::with_slice(self);

        let a = buffer.left_most();
        result.push(a);

        let b = if buffer.is_empty() {
            return result;
        } else {
            buffer.edge::<true>(a)
        };

        result.push(b);

        let c = if buffer.is_empty() {
            return result;
        } else {
            buffer.edge::<false>(a)
        };

        // only one last point is possible
        if !buffer.buffer.is_empty() {
            let d = buffer.buffer[0];
            let bc = b - c;
            let bd = b - d;
            let cross = bd.cross_product(&bc);
            if cross > 0 {
                result.push(d);
            }
        }

        result.push(c);

        result
    }
}

trait Util {
    fn left_most(&mut self) -> IntPoint;
    fn edge<const REVERSED: bool>(&mut self, a: IntPoint) -> IntPoint;
}

impl Util for FourVec<IntPoint> {
    #[inline]
    fn left_most(&mut self) -> IntPoint {
        let mut a = self.buffer[0];
        let mut j = 0;
        let mut i = 1;
        while i < self.len {
            let p = self.buffer[i];
            match p.cmp(&a) {
                Ordering::Less => {
                    j = i;
                    a = p;
                    i += 1;
                }
                Ordering::Greater => i += 1,
                Ordering::Equal => self.remove(i),
            }
        }

        self.extract(j)
    }

    #[inline]
    fn edge<const REVERSED: bool>(&mut self, a: IntPoint) -> IntPoint {
        debug_assert!(!self.is_empty());

        let mut j = 0;
        let mut i = 1;
        let mut b = self.buffer[0];
        let mut e = b - a;

        while i < self.len {
            let p = self.buffer[i];
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
                    self.remove(i);
                    if a.sqr_dist(&b) < a.sqr_dist(&p) {
                        self.buffer[j] = p;
                        b = p;
                        e = v;
                    }
                }
            }
        }

        self.extract(j)
    }
}

#[cfg(test)]
mod tests {
    use crate::int::collision::convex::Convexity;
    use crate::int::collision::four_convex_path::FourConvexPath;
    use crate::int::math::point::IntPoint;
    use rand::Rng;

    #[test]
    fn test_0() {
        let result = [IntPoint::new(0, 0)].to_four_convex();

        assert_eq!(result.len, 1);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_1() {
        let result = [IntPoint::new(0, 0), IntPoint::new(0, 0)].to_four_convex();

        assert_eq!(result.len, 1);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_2() {
        let result = [IntPoint::new(0, 0), IntPoint::new(10, 0)].to_four_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, 0));
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_3() {
        let result = [IntPoint::new(10, 0), IntPoint::new(0, 0)].to_four_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, 0));
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_4() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(10, -10),
            IntPoint::new(10, 10),
        ]
        .to_four_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_5() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]
        .to_four_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_6() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(20, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]
        .to_four_convex();

        assert_eq!(result.len, 4);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(20, 0));
        assert_eq!(result.buffer[3], IntPoint::new(10, 10));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_7() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]
        .to_four_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_8() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]
        .to_four_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_9() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 0),
        ]
        .to_four_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(15, 0));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_10() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 10),
        ]
        .to_four_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, 0));
        assert_eq!(result.buffer[2], IntPoint::new(15, 10));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_11() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(0, 5),
            IntPoint::new(0, 10),
            IntPoint::new(0, 15),
        ]
        .to_four_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(0, 15));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_12() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(0, -5),
            IntPoint::new(0, -10),
            IntPoint::new(0, -15),
        ]
        .to_four_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, -15));
        assert_eq!(result.buffer[1], IntPoint::new(0, 0));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_13() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(10, -10),
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
        ]
        .to_four_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.slice().is_convex());
    }

    #[test]
    fn test_random() {
        let range = -1000i64..=1000i64;
        let mut rng = rand::rng();
        let mut points = [IntPoint::zero(); 4];
        for _ in 0..1000 {
            for i in 0..4 {
                let x = rng.random_range(range.clone());
                let y = rng.random_range(range.clone());
                points[i] = IntPoint::new(x, y);
            }

            let result = points.to_four_convex();
            assert!(result.slice().is_convex());
        }
    }
}
