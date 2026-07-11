use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::normalization::canonical::PushCanonicalSegment;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_shape::int::IntPoint;

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
    //         let a_rect: IntRect<I> = a.ends().into();
    //         for b in b_segments.iter() {
    //             let b_rect: IntRect<I> = b.ends().into();
    //             if a_rect.is_intersect_border_exclude(&b_rect) {
    //
    //             }
    //         }
    //     }
    //     _
    // }
}
