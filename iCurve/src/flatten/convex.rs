use crate::collections::stack_vec::StackVec;
use crate::flatten::segment::{CubicSegment, LineSegment, QuadSegment};
use core::cmp::Ordering;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

pub trait ToIntConvex<P: FloatPointCompatible, I: IntNumber> {
    fn to_int_convex(&self, adapter: FloatPointAdapter<P, I>) -> StackVec<IntPoint<I>, 4>;
}

impl<P: FloatPointCompatible, I: IntNumber> ToIntConvex<P, I> for CubicSegment<P> {
    #[inline]
    fn to_int_convex(&self, adapter: FloatPointAdapter<P, I>) -> StackVec<IntPoint<I>, 4> {
        self.control_points.map(|p| adapter.float_to_int(&p)).to_convex()
    }
}

impl<P: FloatPointCompatible, I: IntNumber> ToIntConvex<P, I> for QuadSegment<P> {
    #[inline]
    fn to_int_convex(&self, adapter: FloatPointAdapter<P, I>) -> StackVec<IntPoint<I>, 4> {
        self.control_points.map(|p| adapter.float_to_int(&p)).to_convex()
    }
}

impl<P: FloatPointCompatible, I: IntNumber> ToIntConvex<P, I> for LineSegment<P> {
    #[inline]
    fn to_int_convex(&self, adapter: FloatPointAdapter<P, I>) -> StackVec<IntPoint<I>, 4> {
        self.control_points.map(|p| adapter.float_to_int(&p)).to_convex()
    }
}

trait ToConvex<I: IntNumber> {
    fn to_convex(&self) -> StackVec<IntPoint<I>, 4>;
}

impl<I: IntNumber> ToConvex<I> for [IntPoint<I>] {
    fn to_convex(&self) -> StackVec<IntPoint<I>, 4> {
        debug_assert!(!self.is_empty());
        debug_assert!(self.len() <= 4);
        let mut result = StackVec::new();
        let mut buffer = StackVec::from_slice(self);

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
            let cross = bd.cross_product(bc);
            if cross > I::Wide::ZERO {
                result.push(d);
            }
        }

        result.push(c);

        result
    }
}

trait Util<I: IntNumber> {
    fn left_most(&mut self) -> IntPoint<I>;
    fn edge<const REVERSED: bool>(&mut self, a: IntPoint<I>) -> IntPoint<I>;
}

impl<I: IntNumber> Util<I> for StackVec<IntPoint<I>, 4> {
    #[inline]
    fn left_most(&mut self) -> IntPoint<I> {
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
                Ordering::Equal => self.swap_remove(i),
            }
        }

        self.swap_extract(j)
    }

    #[inline]
    fn edge<const REVERSED: bool>(&mut self, a: IntPoint<I>) -> IntPoint<I> {
        debug_assert!(!self.is_empty());

        let mut j = 0;
        let mut i = 1;
        let mut b = self.buffer[0];
        let mut e = b - a;

        while i < self.len {
            let p = self.buffer[i];
            let v = p - a;
            let cross = v.cross_product(e);

            let ord = if REVERSED { I::Wide::ZERO.cmp(&cross) } else { cross.cmp(&I::Wide::ZERO) };

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
                    if a.sqr_distance(b) < a.sqr_distance(p) {
                        self.buffer[j] = p;
                        b = p;
                        e = v;
                    }
                }
            }
        }

        self.swap_extract(j)
    }
}

#[cfg(test)]
mod tests {
    use crate::flatten::convex::ToConvex;
    use i_overlay::i_shape::int::IntPoint;
    use rand::RngExt;

    trait Convexity {
        fn is_convex(&self) -> bool;
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
            let sign = e0.cross_product(ei) > 0;

            for &p in self.iter().skip(1) {
                let e = p - pi;
                if (ei.cross_product(e) > 0) != sign {
                    return false;
                }
                pi = p;
                ei = e;
            }

            true
        }
    }

    #[test]
    fn test_0() {
        let result = [IntPoint::ZERO].to_convex();

        assert_eq!(result.len, 1);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_1() {
        let result = [IntPoint::ZERO, IntPoint::ZERO].to_convex();

        assert_eq!(result.len, 1);
        assert_eq!(result.buffer[0], IntPoint::ZERO);
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_2() {
        let result = [IntPoint::new(0, 0), IntPoint::new(10, 0)].to_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, 0));
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_3() {
        let result = [IntPoint::new(10, 0), IntPoint::new(0, 0)].to_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, 0));
        assert!(result.buffer.is_convex());
    }

    #[test]
    fn test_4() {
        let result = [IntPoint::new(0, 0), IntPoint::new(10, -10), IntPoint::new(10, 10)].to_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_5() {
        let result = [IntPoint::new(0, 0), IntPoint::new(10, 10), IntPoint::new(10, -10)].to_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_6() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(20, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]
        .to_convex();

        assert_eq!(result.len, 4);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(20, 0));
        assert_eq!(result.buffer[3], IntPoint::new(10, 10));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_7() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]
        .to_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_8() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]
        .to_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_9() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 0),
        ]
        .to_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(15, 0));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_10() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 10),
        ]
        .to_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, 0));
        assert_eq!(result.buffer[2], IntPoint::new(15, 10));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_11() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(0, 5),
            IntPoint::new(0, 10),
            IntPoint::new(0, 15),
        ]
        .to_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(0, 15));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_12() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(0, -5),
            IntPoint::new(0, -10),
            IntPoint::new(0, -15),
        ]
        .to_convex();

        assert_eq!(result.len, 2);
        assert_eq!(result.buffer[0], IntPoint::new(0, -15));
        assert_eq!(result.buffer[1], IntPoint::new(0, 0));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_13() {
        let result = [
            IntPoint::new(0, 0),
            IntPoint::new(10, -10),
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
        ]
        .to_convex();

        assert_eq!(result.len, 3);
        assert_eq!(result.buffer[0], IntPoint::new(0, 0));
        assert_eq!(result.buffer[1], IntPoint::new(10, -10));
        assert_eq!(result.buffer[2], IntPoint::new(10, 10));
        assert!(result.as_slice().is_convex());
    }

    #[test]
    fn test_random() {
        let range = -1000i32..=1000i32;
        let mut rng = rand::rng();
        let mut points = [IntPoint::ZERO; 4];
        for _ in 0..1000 {
            for i in 0..4 {
                let x = rng.random_range(range.clone());
                let y = rng.random_range(range.clone());
                points[i] = IntPoint::new(x, y);
            }

            let result = points.to_convex();
            assert!(result.as_slice().is_convex());
        }
    }
}
