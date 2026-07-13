use crate::collections::stack_vec::StackVec;
use crate::kernel::int::cross::chord::ChordCross;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::math::angle::ApproximateAngle;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;
use crate::kernel::int::curve::chord::Chord;

pub(crate) struct SplitOptions<I: IntNumber> {
    pub(super) max_parts_count_log: u32,
    pub(super) min_len_pow2: u32,
    pub(super) min_sqr_len_pow2: u32,
    min_separation_log2: u32,
    sin_angle_neg_pow2: u32,
    pub(super) cross_radius: I::Wide,
}

pub(super) struct SegmentIntersector<I: IntNumber> {
    pub(super) options: SplitOptions<I>,
    original_segment_0: Segment<I>,
    original_segment_1: Segment<I>,
}

pub(super) struct Pair<I: IntNumber> {
    s0: Segment<I>,
    ch0: StackVec<IntPoint<I>, 4>,
    t0: SegmentParam<I>,
    step0: SegmentParam<I>,
    is_nearly_linear_0: bool,
    s1: Segment<I>,
    ch1: StackVec<IntPoint<I>, 4>,
    t1: SegmentParam<I>,
    step1: SegmentParam<I>,
    is_nearly_linear_1: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactPoint<I: IntNumber> {
    pub point: IntPoint<I>,
    pub t0: SegmentParam<I>,
    pub t1: SegmentParam<I>,
    pub contact_type: ContactType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactType {
    Cross,
    Tangent,
}

impl<I: IntNumber> SegmentIntersector<I> {
    pub(crate) fn new(s0: Segment<I>, s1: Segment<I>, options: SplitOptions<I>) -> Self {
        Self {
            original_segment_0: s0,
            original_segment_1: s1,
            options,
        }
    }

    pub(crate) fn intersect(&self) -> Vec<ContactPoint<I>> {
        let mut stack = Vec::new();
        let mut output = Vec::new();
        self.intersect_with_buffer(&mut stack, &mut output);
        output
    }

    pub(crate) fn intersect_with_buffer(&self, stack: &mut Vec<Pair<I>>, output: &mut Vec<ContactPoint<I>>) {
        stack.clear();

        let first = Pair {
            s0: self.original_segment_0,
            ch0: self.original_segment_0.convex_hull(),
            t0: SegmentParam::half(),
            step0: SegmentParam::half(),
            is_nearly_linear_0: self
                .original_segment_0
                .is_nearly_linear(self.options.sin_angle_neg_pow2),
            s1: self.original_segment_1,
            ch1: self.original_segment_1.convex_hull(),
            t1: SegmentParam::half(),
            step1: SegmentParam::half(),
            is_nearly_linear_1: self
                .original_segment_1
                .is_nearly_linear(self.options.sin_angle_neg_pow2),
        };

        stack.push(first);

        while let Some(pair) = stack.pop() {
            if pair
                .ch0
                .has_separation_at_least_pow2(&pair.ch1, self.options.min_separation_log2)
            {
                continue;
            }

            let chord0 = pair.s0.chord();
            let chord1 = pair.s1.chord();

            let sqr_len_0 = chord0.sqr_length();
            let sqr_len_1 = chord1.sqr_length();

            let min_sqr_len_log = sqr_len_0.max(sqr_len_1).ilog2();

            if min_sqr_len_log < self.options.min_sqr_len_pow2 {
                if let Some(ChordCross::Point(point)) = chord0.cross(&chord1, self.options.cross_radius) {
                    output.push(ContactPoint {
                        point,
                        t0: global_param(pair.t0, pair.step0, chord0.param_for_point(point)),
                        t1: global_param(pair.t1, pair.step1, chord1.param_for_point(point)),
                        contact_type: ContactType::Cross,
                    });
                }
                continue;
            }

            if pair.is_nearly_linear_0 && pair.is_nearly_linear_1 {
                let v0 = pair.s0.chord().vector();
                let v1 = pair.s1.chord().vector();
                if v0.is_nearly_collinear_with(v1, self.options.sin_angle_neg_pow2) {
                    self.intersect_parallel(
                        pair.s0, pair.t0, pair.step0, pair.s1, pair.t1, pair.step1, output,
                    );
                    continue;
                }
            }

            if sqr_len_0 < sqr_len_1 {
                let chord = pair.s1.chord();
                let (a, b) = (chord.a, chord.b);
                let split = pair.s1.split(pair.t1, pair.step1, a, b);

                if let Some(s1) = split.s0 {
                    let linear =
                        pair.is_nearly_linear_1 || s1.is_nearly_linear(self.options.sin_angle_neg_pow2);
                    stack.push(Pair {
                        s0: pair.s0,
                        ch0: pair.ch0,
                        t0: pair.t0,
                        step0: pair.step0,
                        is_nearly_linear_0: pair.is_nearly_linear_0,
                        s1,
                        ch1: s1.convex_hull(),
                        t1: split.t0,
                        step1: split.step,
                        is_nearly_linear_1: linear,
                    });
                }

                if let Some(s1) = split.s1 {
                    let linear =
                        pair.is_nearly_linear_1 || s1.is_nearly_linear(self.options.sin_angle_neg_pow2);
                    stack.push(Pair {
                        s0: pair.s0,
                        ch0: pair.ch0,
                        t0: pair.t0,
                        step0: pair.step0,
                        is_nearly_linear_0: pair.is_nearly_linear_0,
                        s1,
                        ch1: s1.convex_hull(),
                        t1: split.t1,
                        step1: split.step,
                        is_nearly_linear_1: linear,
                    });
                }
            } else {
                let chord = pair.s0.chord();
                let (a, b) = (chord.a, chord.b);
                let split = pair.s0.split(pair.t0, pair.step0, a, b);

                if let Some(s0) = split.s0 {
                    let linear =
                        pair.is_nearly_linear_0 || s0.is_nearly_linear(self.options.sin_angle_neg_pow2);

                    stack.push(Pair {
                        s0,
                        ch0: s0.convex_hull(),
                        t0: split.t0,
                        step0: split.step,
                        is_nearly_linear_0: linear,
                        s1: pair.s1,
                        ch1: pair.ch1,
                        t1: pair.t1,
                        step1: pair.step1,
                        is_nearly_linear_1: pair.is_nearly_linear_1,
                    });
                }

                if let Some(s0) = split.s1 {
                    let linear =
                        pair.is_nearly_linear_0 || s0.is_nearly_linear(self.options.sin_angle_neg_pow2);
                    stack.push(Pair {
                        s0,
                        ch0: s0.convex_hull(),
                        t0: split.t1,
                        step0: split.step,
                        is_nearly_linear_0: linear,
                        s1: pair.s1,
                        ch1: pair.ch1,
                        t1: pair.t1,
                        step1: pair.step1,
                        is_nearly_linear_1: pair.is_nearly_linear_1,
                    });
                }
            }
        }
    }
}

#[inline]
pub(super) fn global_param<I: IntNumber>(
    center: SegmentParam<I>,
    step: SegmentParam<I>,
    local: SegmentParam<I>,
) -> SegmentParam<I> {
    let denominator = SegmentParam::<I>::DENOMINATOR;
    let start = center.value() - step.value();
    let span = step.value() << 1;
    let offset = (span * local.value() + (denominator >> 1)) / denominator;
    SegmentParam::new(I::from_wide(start + offset))
}

impl<I: IntNumber> Default for SplitOptions<I> {
    fn default() -> Self {
        let min_len_pow2 = 4;
        Self {
            max_parts_count_log: 6,
            min_len_pow2,
            min_sqr_len_pow2: 2 * min_len_pow2,
            min_separation_log2: 2,
            sin_angle_neg_pow2: 3,
            cross_radius: I::Wide::TWO,
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
        let s0 = Segment::Cubic(CubicSegment {
            control_points: [
                [100, 100].into(),
                [100, 400].into(),
                [600, 900].into(),
                [1000, 900].into(),
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

        let intersector = SegmentIntersector::new(s0, s1, SplitOptions::default());
        let result = intersector.intersect();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].contact_type, super::ContactType::Cross);
    }
}
