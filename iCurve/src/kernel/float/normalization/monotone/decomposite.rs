use crate::collections::stack_vec::StackVec;
use crate::kernel::float::curve::param::FloatSegmentParam;
use crate::kernel::float::curve::split_at::FloatSplitAt;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MonotoneDecompositionDirection {
    X,
    Y,
}

pub(crate) trait DecompositeIntoMonotone {
    type Output;
    fn decomposite_into_monotone(&self) -> Self::Output;
}
#[inline]
pub(super) fn roots_to_segments<T, S, const ROOT_CAP: usize, const SEGMENT_CAP: usize>(
    segment: &S,
    mut roots: StackVec<FloatSegmentParam<T>, ROOT_CAP>,
) -> StackVec<S, SEGMENT_CAP>
where
    T: FloatNumber,
    S: FloatSplitAt<T, Output = [S; 2]> + Copy + Default,
{
    sort_and_dedup_roots(&mut roots);

    let mut output = StackVec::new();
    let mut t0 = FloatSegmentParam::Start;

    for &t1 in roots.as_slice() {
        output.push(segment_range(segment, t0, t1));
        t0 = t1;
    }

    output.push(segment_range(segment, t0, FloatSegmentParam::End));
    output
}

#[inline]
fn segment_range<T, S>(segment: &S, t0: FloatSegmentParam<T>, t1: FloatSegmentParam<T>) -> S
where
    T: FloatNumber,
    S: FloatSplitAt<T, Output = [S; 2]> + Copy,
{
    if t0 == FloatSegmentParam::Start && t1 == FloatSegmentParam::End {
        return *segment;
    }

    if t0 == FloatSegmentParam::Start {
        let [range, _] = segment.split_at(t1.value());
        return range;
    }

    let [_, right] = segment.split_at(t0.value());
    if t1 == FloatSegmentParam::End {
        return right;
    }

    let t0 = t0.value();
    let local_t = (t1.value() - t0) / (T::ONE - t0);
    let [range, _] = right.split_at(local_t);
    range
}

fn sort_and_dedup_roots<T: FloatNumber, const CAP: usize>(roots: &mut StackVec<FloatSegmentParam<T>, CAP>) {
    roots.buffer[0..roots.len].sort_unstable();

    let mut write_index = 0;
    let mut previous = FloatSegmentParam::Start;

    for read_index in 0..roots.len {
        let root = roots.buffer[read_index];
        if root == previous {
            continue;
        }

        roots.buffer[write_index] = root;
        write_index += 1;
        previous = root;
    }

    roots.len = write_index;
}
