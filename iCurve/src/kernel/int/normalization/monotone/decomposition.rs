use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::split_at::{SplitAt, segment_range};
use i_overlay::i_float::int::number::int::IntNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MonotoneDecompositionDirection {
    X,
    Y,
}

pub(crate) trait DecomposeIntoMonotone {
    type Output;
    fn decompose_into_monotone(&self) -> Self::Output;
}

#[inline]
pub(crate) fn roots_to_segments<I, S, const ROOT_CAP: usize, const SEGMENT_CAP: usize>(
    segment: &S,
    mut roots: StackVec<SegmentParam<I>, ROOT_CAP>,
) -> StackVec<S, SEGMENT_CAP>
where
    I: IntNumber,
    S: SplitAt<I, Output = [S; 2]> + Chord<I> + Copy + Default,
{
    roots.buffer[0..roots.len].sort_unstable_by_key(|root| root.value());
    roots.dedup();

    let mut output = StackVec::new();
    let mut t0 = SegmentParam::new(I::ZERO);

    for &t1 in roots.as_slice() {
        let s = segment_range(segment, t0, t1);
        if !s.chord().is_zero_length() {
            output.push(s);
        }
        t0 = t1;
    }

    let s = segment_range(
        segment,
        t0,
        SegmentParam::new(I::from_wide(SegmentParam::<I>::DENOMINATOR)),
    );

    if !s.chord().is_zero_length() {
        output.push(s);
    }

    output
}
