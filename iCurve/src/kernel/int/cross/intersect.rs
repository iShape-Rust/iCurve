use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::normalization::canonical::PushCanonicalSegment;

pub struct Intersection<I: IntNumber> {
    pub points: Vec<IntPoint<I>>,
    pub overlaps: Vec<Segment<I>>,
}

impl<I: IntNumber> Segment<I> {
    // pub fn intersect(self, other: Self) -> Intersection<I> {
    //     let mut a_segments = Vec::new();
    //     let mut b_segments = Vec::new();
    //
    //     a_segments.push_canonical(self);
    //     b_segments.push_canonical(other);
    //
    //     for a in a_segments.iter() {
    //         for b in b_segments.iter() {
    //
    //         }
    //     }
    //
    //     _
    // }

}