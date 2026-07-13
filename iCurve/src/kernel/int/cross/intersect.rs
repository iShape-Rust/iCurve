use crate::kernel::int::curve::segment::Segment;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use crate::kernel::int::cross::intersector::{ContactPoint, SegmentIntersector, SplitOptions};
use crate::kernel::int::normalization::canonical::PushCanonicalSegment;

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
