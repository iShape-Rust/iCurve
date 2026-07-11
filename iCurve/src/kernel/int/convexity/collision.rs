use crate::collections::stack_vec::StackVec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

impl<I: IntNumber> StackVec<IntPoint<I>, 4> {
    #[inline]
    pub(crate) fn is_overlapping_border_excluded(&self, other: &Self) -> bool {
        !self.has_separate_line::<true>(other.as_slice()) && !other.has_separate_line::<true>(self.as_slice())
    }

    #[inline]
    pub(crate) fn is_overlapping_border_included(&self, other: &Self) -> bool {
        !self.has_separate_line::<false>(other.as_slice())
            && !other.has_separate_line::<false>(self.as_slice())
    }

    #[inline]
    pub(crate) fn are_separated_from_each_other_by_at_least_a_distance(&self, other: &Self, min_dist_log: u32) -> bool {
        !self.has_separate_line_not_thinner_then(other.as_slice(), min_dist_log)
            && !other.has_separate_line_not_thinner_then(self.as_slice(), min_dist_log)
    }

    fn has_separate_line<const INCLUDE_BORDER: bool>(&self, points: &[IntPoint<I>]) -> bool {
        let inside_limit = if INCLUDE_BORDER {
            I::Wide::ZERO
        } else {
            I::Wide::ONE
        };
        let mut a = *self.as_slice().last().unwrap();

        'main_loop: for &b in self.as_slice().iter() {
            let ba = b - a;
            for &p in points.iter() {
                let ap = a - p;
                let cross = ba.cross_product(ap);
                if cross < inside_limit {
                    a = b;
                    continue 'main_loop;
                }
            }
            return true;
        }

        false
    }

    fn has_separate_line_not_thinner_then(&self, points: &[IntPoint<I>], min_dist_log: u32) -> bool {
        let mut a = *self.as_slice().last().unwrap();

        'main_loop: for &b in self.as_slice().iter() {
            let ba = b - a;
            let log_ba = ba.sqr_length().ilog2() / 2;

            for &p in points.iter() {
                let ap = a - p;
                let cross = ba.cross_product(ap);
                if cross <= I::Wide::ZERO {
                    a = b;
                    continue 'main_loop;
                }

                let log_cross = cross.ilog2();
                let log_dist = log_cross.saturating_sub(log_ba);

                if log_dist <= min_dist_log {
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
fn test_overlapping_0() {
    let convex_0 = StackVec::with_slice_as_convex(&[
        IntPoint::new(0, 0),
        IntPoint::new(10, 0),
        IntPoint::new(10, 10),
        IntPoint::new(0, 10),
    ]);

    let convex_1 =
        StackVec::with_slice_as_convex(&[IntPoint::new(10, 5), IntPoint::new(20, 0), IntPoint::new(20, 10)]);

    assert!(!convex_0.is_overlapping_border_excluded(&convex_1));
    assert!(!convex_1.is_overlapping_border_excluded(&convex_0));
    assert!(convex_0.is_overlapping_border_included(&convex_1));
    assert!(convex_1.is_overlapping_border_included(&convex_0));
}

#[test]
fn test_overlapping_1() {
    let convex_0 = StackVec::with_slice_as_convex(&[
        IntPoint::new(0, 0),
        IntPoint::new(10, 0),
        IntPoint::new(10, 10),
        IntPoint::new(0, 10),
    ]);

    let convex_1 =
        StackVec::with_slice_as_convex(&[IntPoint::new(11, 5), IntPoint::new(20, 0), IntPoint::new(20, 10)]);

    assert!(!convex_0.is_overlapping_border_excluded(&convex_1));
    assert!(!convex_1.is_overlapping_border_excluded(&convex_0));
    assert!(!convex_0.is_overlapping_border_included(&convex_1));
    assert!(!convex_1.is_overlapping_border_included(&convex_0));
}

#[cfg(test)]
mod tests {
    use crate::collections::stack_vec::StackVec;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn test_overlapping_2() {
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

        assert!(convex_0.is_overlapping_border_excluded(&convex_1));
        assert!(convex_1.is_overlapping_border_excluded(&convex_0));
        assert!(convex_0.is_overlapping_border_included(&convex_1));
        assert!(convex_1.is_overlapping_border_included(&convex_0));
    }

    #[test]
    fn test_overlapping_3() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]);

        // self contains
        assert!(convex.is_overlapping_border_excluded(&convex));
        assert!(convex.is_overlapping_border_included(&convex));
    }

    #[test]
    fn test_has_separate_line_with_margin_0() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(4, 0),
            IntPoint::new(4, 4),
            IntPoint::new(0, 4),
        ]);

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-1, 0)], 1));
        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(0, 0)], 1));
        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(1, 2)], 1));
        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 0)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-4, 0)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-7, 0)], 2));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-8, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_1() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(5, 5),
            IntPoint::new(0, 5),
        ]);

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 0)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-4, 0)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 2)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-4, 2)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-6, 0)], 2));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-7, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_2() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(6, 0),
            IntPoint::new(6, 6),
            IntPoint::new(0, 6),
        ]);

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-2, 0)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 0)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-2, 4)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 4)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-5, 0)], 2));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-6, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_3() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(7, 0),
            IntPoint::new(7, 7),
            IntPoint::new(0, 7),
        ]);

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-2, 0)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 0)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-2, 5)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 5)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-4, 0)], 2));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-5, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_4() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(8, 0),
            IntPoint::new(8, 8),
            IntPoint::new(0, 8),
        ]);

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 0)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-4, 0)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-3, 8)], 1));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-4, 8)], 1));

        assert!(!convex.has_separate_line_not_thinner_then(&[IntPoint::new(-7, 0)], 2));
        assert!(convex.has_separate_line_not_thinner_then(&[IntPoint::new(-8, 0)], 2));
    }
}
