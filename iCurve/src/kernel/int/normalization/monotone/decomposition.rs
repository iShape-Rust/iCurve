use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::split_at::SplitAt;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

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
pub(super) fn roots_to_segments<I, S, const ROOT_CAP: usize, const SEGMENT_CAP: usize>(
    segment: &S,
    mut roots: StackVec<SegmentParam<I>, ROOT_CAP>,
) -> StackVec<S, SEGMENT_CAP>
where
    I: IntNumber,
    S: SplitAt<I, Output = [S; 2]> + Copy + Default,
{
    sort_and_dedup_roots(&mut roots);

    let mut output = StackVec::new();
    let mut t0 = SegmentParam::new(I::ZERO);

    for &t1 in roots.as_slice() {
        output.push(segment_range(segment, t0, t1));
        t0 = t1;
    }

    output.push(segment_range(
        segment,
        t0,
        SegmentParam::new(I::from_wide(SegmentParam::<I>::DENOMINATOR)),
    ));
    output
}

#[inline]
fn segment_range<I, S>(segment: &S, t0: SegmentParam<I>, t1: SegmentParam<I>) -> S
where
    I: IntNumber,
    S: SplitAt<I, Output = [S; 2]> + Copy,
{
    if t0.value() == I::Wide::ZERO && t1.value() == SegmentParam::<I>::DENOMINATOR {
        return *segment;
    }

    if t0.value() == I::Wide::ZERO {
        return segment.split_at_left(t1);
    }

    let right = segment.split_at_right(t0);
    if t1.value() == SegmentParam::<I>::DENOMINATOR {
        return right;
    }

    let numerator = t1.value() - t0.value();
    let denominator = SegmentParam::<I>::DENOMINATOR - t0.value();
    let local_t = SegmentParam::from_int(I::from_wide(numerator), I::from_wide(denominator));

    right.split_at_left(local_t)
}

fn sort_and_dedup_roots<I: IntNumber, const CAP: usize>(roots: &mut StackVec<SegmentParam<I>, CAP>) {
    roots.buffer[0..roots.len].sort_unstable_by_key(|root| root.value());

    let mut write_index = 0;
    let mut previous = I::Wide::ZERO;

    for read_index in 0..roots.len {
        let root = roots.buffer[read_index];
        if root.value() == previous {
            continue;
        }

        roots.buffer[write_index] = root;
        write_index += 1;
        previous = root.value();
    }

    roots.len = write_index;
}
