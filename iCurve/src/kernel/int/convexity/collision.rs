use crate::collections::stack_vec::StackVec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

impl<I: IntNumber> StackVec<IntPoint<I>, 4> {
    pub(crate) fn contains_point_border_included(&self, point: IntPoint<I>) -> bool {
        let points = self.as_slice();
        match points {
            [] => false,
            [a] => *a == point,
            [a, b] => {
                let ab = *b - *a;
                let ap = point - *a;
                ab.cross_product(ap) == I::Wide::ZERO
                    && a.x.min(b.x) <= point.x
                    && point.x <= a.x.max(b.x)
                    && a.y.min(b.y) <= point.y
                    && point.y <= a.y.max(b.y)
            }
            _ => {
                let mut has_positive = false;
                let mut has_negative = false;
                let mut a = *points.last().unwrap();

                for &b in points {
                    let cross = (b - a).cross_product(point - a);
                    has_positive |= cross > I::Wide::ZERO;
                    has_negative |= cross < I::Wide::ZERO;
                    if has_positive && has_negative {
                        return false;
                    }
                    a = b;
                }

                true
            }
        }
    }

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
    pub(crate) fn has_separation_at_least_pow2(&self, other: &Self, min_separation_log2: u32) -> bool {
        self.has_separating_edge_at_least_pow2(other.as_slice(), min_separation_log2)
            || other.has_separating_edge_at_least_pow2(self.as_slice(), min_separation_log2)
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

    fn has_separating_edge_at_least_pow2(&self, points: &[IntPoint<I>], min_separation_log2: u32) -> bool {
        let mut a = *self.as_slice().last().unwrap();

        'main_loop: for &b in self.as_slice().iter() {
            let ba = b - a;
            debug_assert!(b != a);
            let log_ba = ba.sqr_length().ilog2() / 2;

            for &p in points.iter() {
                let ap = a - p;
                let cross = ba.cross_product(ap);
                if cross <= I::Wide::ZERO {
                    a = b;
                    continue 'main_loop;
                }

                let log_cross = cross.ilog2();
                if log_cross <= log_ba || log_cross - log_ba <= min_separation_log2 {
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
    use i_overlay::i_shape::int_path;

    #[test]
    fn contains_point_with_border_for_polygon_and_line_hulls() {
        let polygon = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(10, 0),
            IntPoint::new(10, 10),
            IntPoint::new(0, 10),
        ]);
        assert!(polygon.contains_point_border_included(IntPoint::new(5, 5)));
        assert!(polygon.contains_point_border_included(IntPoint::new(0, 5)));
        assert!(!polygon.contains_point_border_included(IntPoint::new(-1, 5)));

        let line = StackVec::with_slice_as_convex(&[IntPoint::new(0, 0), IntPoint::new(10, 10)]);
        assert!(line.contains_point_border_included(IntPoint::new(5, 5)));
        assert!(!line.contains_point_border_included(IntPoint::new(5, 6)));
    }

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

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-1, 0)], 1));
        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(0, 0)], 1));
        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(1, 2)], 1));
        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 0)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-4, 0)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-7, 0)], 2));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-8, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_1() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(5, 0),
            IntPoint::new(5, 5),
            IntPoint::new(0, 5),
        ]);

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 0)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-4, 0)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 2)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-4, 2)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-6, 0)], 2));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-7, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_2() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(6, 0),
            IntPoint::new(6, 6),
            IntPoint::new(0, 6),
        ]);

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-2, 0)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 0)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-2, 4)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 4)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-5, 0)], 2));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-6, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_3() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(7, 0),
            IntPoint::new(7, 7),
            IntPoint::new(0, 7),
        ]);

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-2, 0)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 0)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-2, 5)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 5)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-4, 0)], 2));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-5, 0)], 2));
    }

    #[test]
    fn test_has_separate_line_with_margin_4() {
        let convex = StackVec::with_slice_as_convex(&[
            IntPoint::new(0, 0),
            IntPoint::new(8, 0),
            IntPoint::new(8, 8),
            IntPoint::new(0, 8),
        ]);

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 0)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-4, 0)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-3, 8)], 1));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-4, 8)], 1));

        assert!(!convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-7, 0)], 2));
        assert!(convex.has_separating_edge_at_least_pow2(&[IntPoint::new(-8, 0)], 2));
    }

    #[test]
    fn test_has_separation_at_least_pow2_0() {
        let c0 = StackVec::with_slice_as_convex(&int_path![[0, 0], [8, 0], [8, 8], [0, 8]]);
        let c1 = StackVec::with_slice_as_convex(&int_path![[10, 5], [15, 0], [20, 5], [15, 10]]);

        assert!(!c0.has_separation_at_least_pow2(&c1, 1));
        assert!(!c0.has_separation_at_least_pow2(&c1, 2));
        assert!(!c0.has_separation_at_least_pow2(&c1, 3));
        assert!(!c0.has_separation_at_least_pow2(&c1, 4));
    }

    #[test]
    fn test_has_separation_at_least_pow2_1() {
        let c0 = StackVec::with_slice_as_convex(&int_path![[0, 0], [8, 0], [8, 8], [0, 8]]);
        let c1 = StackVec::with_slice_as_convex(&int_path![[20, 5], [25, 0], [30, 5], [25, 10]]);

        assert!(c0.has_separation_at_least_pow2(&c1, 1));
        assert!(c0.has_separation_at_least_pow2(&c1, 2));
        assert!(!c0.has_separation_at_least_pow2(&c1, 3));
        assert!(!c0.has_separation_at_least_pow2(&c1, 4));
    }
}
