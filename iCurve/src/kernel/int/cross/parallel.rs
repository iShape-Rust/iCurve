use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use crate::kernel::int::cross::intersector::{ContactPoint, SegmentIntersector, SplitOptions};
use crate::kernel::int::curve::segment::Segment;

impl<I: IntNumber> SegmentIntersector<I> {
    pub(super) fn intersect_parallel(&self, s0: Segment<I>, s1: Segment<I>, output: &mut Vec<ContactPoint<I>>) {
        // TODO not implemented yet
    }
}