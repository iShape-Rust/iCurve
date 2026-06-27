use crate::collections::stack_vec::StackVec;
use crate::kernel::float::curve::segment::FloatSegment;
use core::cmp::Ordering;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

impl<I: IntNumber> StackVec<IntPoint<I>, 4> {
    pub(crate) fn with_slice_as_convex(points: &[IntPoint<I>]) -> Self {
        let mut convex: StackVec<IntPoint<I>, 4> = StackVec::new();
        convex.init_as_convex(points);
        convex
    }

    pub(crate) fn with_float_slice_as_convex<T: FloatNumber>(
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        points: &[FloatPoint<T>],
    ) -> Self {
        debug_assert!(!points.is_empty());
        debug_assert!(points.len() <= 4);

        let mut int_points: StackVec<IntPoint<I>, 4> = StackVec::new();
        for point in points {
            int_points.push(adapter.float_to_int(point));
        }

        Self::with_slice_as_convex(int_points.as_slice())
    }

    pub(crate) fn init_as_convex(&mut self, points: &[IntPoint<I>]) {
        debug_assert!(!points.is_empty());
        debug_assert!(points.len() <= 4);
        self.clear();

        let mut buffer = StackVec::from_slice(points);

        let a = buffer.left_most();
        self.push(a);

        let b = if buffer.is_empty() {
            return;
        } else {
            buffer.edge::<true>(a)
        };

        self.push(b);

        let c = if buffer.is_empty() {
            return;
        } else {
            buffer.edge::<false>(a)
        };

        // only one last point is possible
        if !buffer.is_empty() {
            let d = buffer.buffer[0];
            let bc = b - c;
            let bd = b - d;
            let cross = bd.cross_product(bc);
            if cross > I::Wide::ZERO {
                self.push(d);
            }
        }

        self.push(c);
    }

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

            let ord = if REVERSED {
                I::Wide::ZERO.cmp(&cross)
            } else {
                cross.cmp(&I::Wide::ZERO)
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

    #[inline]
    pub(crate) fn is_intersect_border_exclude(&self, other: &Self) -> bool {
        self.has_separate_line(other.as_slice()) || other.has_separate_line(self.as_slice())
    }

    fn has_separate_line(&self, points: &[IntPoint<I>]) -> bool {
        let mut a = *self.as_slice().last().unwrap();

        'main_loop: for &b in self.as_slice().iter() {
            let ba = b - a;
            for &p in points.iter() {
                let ap = a - p;
                let cross = ba.cross_product(ap);
                if cross <= I::Wide::ZERO {
                    a = b;
                    continue 'main_loop;
                }
            }
            return true;
        }

        false
    }
}

impl<T: FloatNumber> FloatSegment<T> {
    pub(crate) fn to_convex<I: IntNumber>(
        &self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> StackVec<IntPoint<I>, 4> {
        match self {
            FloatSegment::Line(line) => StackVec::with_float_slice_as_convex(adapter, &line.control_points),
            FloatSegment::Quad(quad) => StackVec::with_float_slice_as_convex(adapter, &quad.control_points),
            FloatSegment::Cubic(cubic) => {
                StackVec::with_float_slice_as_convex(adapter, &cubic.control_points)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::stack_vec::StackVec;
    use i_overlay::i_shape::int::IntPoint;
    use i_overlay::i_shape::int::path::ContourExtension;
    use rand::RngExt;

    #[test]
    fn test_0() {
        let convex = StackVec::with_slice_as_convex(&[IntPoint::new(0, 0)]);

        assert_eq!(convex.len, 1);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_1() {
        let convex = StackVec::with_slice_as_convex(&[IntPoint::new(0, 0), IntPoint::new(0, 0)]);

        assert_eq!(convex.len, 1);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_2() {
        let convex = StackVec::with_slice_as_convex(&[IntPoint::new(0, 0), IntPoint::new(10, 0)]);

        assert_eq!(convex.len, 2);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, 0));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_3() {
        let convex = StackVec::with_slice_as_convex(&[IntPoint::new(10, 0), IntPoint::new(0, 0)]);

        assert_eq!(convex.len, 2);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, 0));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_4() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, -10),
            IntPoint::new(10, 10),
        ]);

        assert_eq!(convex.len, 3);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, -10));
        assert_eq!(convex.buffer[2], IntPoint::new(10, 10));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_5() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(convex.len, 3);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, -10));
        assert_eq!(convex.buffer[2], IntPoint::new(10, 10));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_6() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(20, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(convex.len, 4);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, -10));
        assert_eq!(convex.buffer[2], IntPoint::new(20, 0));
        assert_eq!(convex.buffer[3], IntPoint::new(10, 10));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_7() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(convex.len, 3);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, -10));
        assert_eq!(convex.buffer[2], IntPoint::new(10, 10));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_8() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 10),
            IntPoint::new(10, -10),
        ]);

        assert_eq!(convex.len, 3);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, -10));
        assert_eq!(convex.buffer[2], IntPoint::new(10, 10));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_9() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 0),
        ]);

        assert_eq!(convex.len, 2);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(15, 0));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_10() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(10, 0),
            IntPoint::new(15, 10),
        ]);

        assert_eq!(convex.len, 3);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, 0));
        assert_eq!(convex.buffer[2], IntPoint::new(15, 10));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_11() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(0, 5),
            IntPoint::new(0, 10),
            IntPoint::new(0, 15),
        ]);

        assert_eq!(convex.len, 2);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(0, 15));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_12() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(0, -5),
            IntPoint::new(0, -10),
            IntPoint::new(0, -15),
        ]);

        assert_eq!(convex.len, 2);
        assert_eq!(convex.buffer[0], IntPoint::new(0, -15));
        assert_eq!(convex.buffer[1], IntPoint::new(0, 0));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_13() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, -10),
            IntPoint::new(5, 5),
            IntPoint::new(10, 10),
        ]);

        assert_eq!(convex.len, 3);
        assert_eq!(convex.buffer[0], IntPoint::new(0, 0));
        assert_eq!(convex.buffer[1], IntPoint::new(10, -10));
        assert_eq!(convex.buffer[2], IntPoint::new(10, 10));
        assert!(convex.as_slice().is_convex());
    }

    #[test]
    fn test_random() {
        let range = -1000i64..=1000i64;
        let mut rng = rand::rng();
        let mut points = [IntPoint::ZERO; 4];
        for _ in 0..1000 {
            for i in 0..4 {
                let x = rng.random_range(range.clone());
                let y = rng.random_range(range.clone());
                points[i] = IntPoint::new(x, y);
            }
            let convex = StackVec::with_slice_as_convex(&points);
            assert!(convex.as_slice().is_convex());
        }
    }

    #[test]
    fn test_overlaps_0() {
        let convex_0 = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]);

        let convex_1 = StackVec::with_slice_as_convex(&[
            IntPoint::new(10, 5),
            IntPoint::new(20, 0),
            IntPoint::new(20, 10),
        ]);

        assert!(!convex_0.is_intersect_border_exclude(&convex_1));
        assert!(!convex_1.is_intersect_border_exclude(&convex_0));
    }

    #[test]
    fn test_overlaps_1() {
        let convex_0 = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]);

        let convex_1 = StackVec::with_slice_as_convex(&[
            IntPoint::new(11, 5),
            IntPoint::new(20, 0),
            IntPoint::new(20, 10),
        ]);

        assert!(convex_0.is_intersect_border_exclude(&convex_1));
        assert!(convex_1.is_intersect_border_exclude(&convex_0));
    }

    #[test]
    fn test_overlaps_2() {
        let convex_0 = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]);

        let convex_1 = StackVec::with_slice_as_convex(&[
            IntPoint::new(9, 5),
            IntPoint::new(20, 0),
            IntPoint::new(20, 10),
        ]);

        assert!(!convex_0.is_intersect_border_exclude(&convex_1));
        assert!(!convex_1.is_intersect_border_exclude(&convex_0));
    }

    #[test]
    fn test_overlaps_3() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]);

        // self contains
        assert!(!convex.is_intersect_border_exclude(&convex));
    }
}
