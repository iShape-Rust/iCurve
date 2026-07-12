use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;
use crate::collections::stack_vec::StackVec;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::math::angle::ApproximateAngle;

pub(crate) struct SplitOptions<I: IntNumber> {
    min_sqr_len: I::Wide,
    min_separation_log2: u32,
    sin_angle_neg_pow2: u32,
}

pub(super) struct SegmentIntersector<I: IntNumber> {
    options: SplitOptions<I>,
    original_segment_0: Segment<I>,
    original_segment_1: Segment<I>
}

struct Pair<I: IntNumber> {
    s0: Segment<I>,
    ch0: StackVec<IntPoint<I>, 4>,
    t0: SegmentParam<I>,
    s1: Segment<I>,
    ch1: StackVec<IntPoint<I>, 4>,
    t1: SegmentParam<I>,
}

pub(crate) struct Intersection {

}

impl<I: IntNumber> SegmentIntersector<I> {

    pub(crate) fn new(s0: Segment<I>, s1: Segment<I>, options: SplitOptions<I>) -> Self {
        Self { original_segment_0: s0, original_segment_1: s1, options }
    }

    pub(crate) fn intersect(&self) {
        let mut stack = Vec::new();
        self.intersect_with_buffer(&mut stack);
    }

    fn intersect_with_buffer(&self, stack: &mut Vec<Pair<I>>) -> Intersection {
        stack.clear();

        let first = Pair {
            s0: self.original_segment_0,
            ch0: self.original_segment_0.convex_hull(),
            t0: SegmentParam::half(),
            s1: self.original_segment_1,
            ch1: self.original_segment_1.convex_hull(),
            t1: SegmentParam::half(),
        };

        stack.push(first);

        while let Some(pair) = stack.pop() {
            if pair.ch0.has_separation_at_least_pow2(&pair.ch1, self.options.min_separation_log2) {
                continue;
            }

            let v0 = pair.s0.base_vector();
            let v1 = pair.s1.base_vector();

            let sqr_len_0 = v0.sqr_length();
            let sqr_len_1 = v1.sqr_length();

            let min_sqr_len = sqr_len_0.min(sqr_len_1);

            if min_sqr_len < self.options.min_sqr_len {
                // TODO finish as line x line intersection
            }

            if v0.is_nearly_collinear_with(v1, self.options.sin_angle_neg_pow2) {
                // TODO finish as near parallel case
            }

            if sqr_len_0 < sqr_len_1 {
                let [a, b] = pair.s1.ends();
                let split = pair.s1.split(pair.t1, a, b);
                stack.push(Pair {
                    s0: pair.s0,
                    ch0: pair.ch0,
                    t0: pair.t0,
                    s1: split.s0,
                    ch1: split.s0.convex_hull(),
                    t1: split.t0,
                });
                stack.push(Pair {
                    s0: pair.s0,
                    ch0: pair.ch0,
                    t0: pair.t0,
                    s1: split.s1,
                    ch1: split.s1.convex_hull(),
                    t1: split.t1,
                });
            } else {
                let [a, b] = pair.s0.ends();
                let split = pair.s1.split(pair.t0, a, b);
                stack.push(Pair {
                    s0: split.s0,
                    ch0: split.s0.convex_hull(),
                    t0: split.t0,
                    s1: pair.s1,
                    ch1: pair.ch1,
                    t1: pair.t1,
                });
                stack.push(Pair {
                    s0: split.s1,
                    ch0: split.s1.convex_hull(),
                    t0: split.t1,
                    s1: pair.s1,
                    ch1: pair.ch1,
                    t1: pair.t1,
                });
            }
        }

        Intersection {}
    }
}

impl<I: IntNumber> Default for SplitOptions<I> {
    fn default() -> Self {
        Self {
            min_sqr_len: I::Wide::from_usize(256),
            min_separation_log2: 2,
            sin_angle_neg_pow2: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::int::cross::intersector::{SegmentIntersector, SplitOptions};
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::segment::Segment;

    #[test]
    fn test_0() {
        let s0 = Segment::Cubic(CubicSegment { control_points: [
            [100, 100].into(), [100, 400].into(), [600, 900].into(), [1000, 900].into()]
        });
        let s1 = Segment::Cubic(CubicSegment { control_points:[
            [100, 900].into(), [100, 500].into(), [600, 0].into(), [1000, 0].into()]
        });

        let intersector = SegmentIntersector::new(s0, s1, SplitOptions::default());
        let result = intersector.intersect();
    }
}