use crate::kernel::int::cross::intersector::{ContactPoint, SegmentIntersector, SplitOptions};
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::normalization::canonical::PushCanonicalSegment;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;

impl<I: IntNumber> Segment<I> {
    pub fn intersect(self, other: Self) -> Vec<ContactPoint<I>> {
        let mut a_segments = Vec::new();
        let mut b_segments = Vec::new();

        a_segments.push_canonical(self);
        b_segments.push_canonical(other);

        let mut stack = Vec::new();
        let mut output = Vec::new();

        for a in a_segments.iter() {
            let a_rect = a.chord().to_rect();
            for b in b_segments.iter() {
                let b_rect = b.chord().to_rect();
                if a_rect.is_intersect_border_exclude(&b_rect) {
                    let intersector = SegmentIntersector::new(*a, *b, SplitOptions::default());
                    intersector.intersect_with_buffer(&mut stack, &mut output);
                }
            }
        }

        output
    }
}
#[cfg(test)]
mod tests {
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::segment::Segment;

    #[test]
    fn test_0() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [940, 899].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_1() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [900, 700].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_2() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [596, 795].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_3() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [597, 795].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_4() {
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [1263, 176].into(),
            ],
        });
        let s1 = Segment::Cubic(CubicSegment {
            control_points: [
                [100i32, 900].into(),
                [100, 500].into(),
                [600, 0].into(),
                [1000, 0].into(),
            ],
        });

        let result = s0.intersect(s1);
        assert_eq!(result.len(), 1);
    }
}
