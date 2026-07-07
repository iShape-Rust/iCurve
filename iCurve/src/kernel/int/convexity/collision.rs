use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;
use crate::collections::stack_vec::StackVec;

impl<I: IntNumber> StackVec<IntPoint<I>, 4> {
    #[inline]
    pub(crate) fn is_overlap_border_exclude(&self, other: &Self) -> bool {
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

    assert!(!convex_0.is_overlap_border_exclude(&convex_1));
    assert!(!convex_1.is_overlap_border_exclude(&convex_0));
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

    assert!(convex_0.is_overlap_border_exclude(&convex_1));
    assert!(convex_1.is_overlap_border_exclude(&convex_0));
}

#[cfg(test)]
mod tests {
    use i_overlay::i_shape::int::IntPoint;
    use crate::collections::stack_vec::StackVec;

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

        assert!(!convex_0.is_overlap_border_exclude(&convex_1));
        assert!(!convex_1.is_overlap_border_exclude(&convex_0));
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
        assert!(!convex.is_overlap_border_exclude(&convex));
    }
}