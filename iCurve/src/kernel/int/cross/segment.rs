use crate::kernel::int::curve::bisect::Bisect;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::math::angle::ApproximateAngle;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

pub(super) struct Split<I: IntNumber> {
    pub(super) t0: SegmentParam<I>,
    pub(super) s0: Option<Segment<I>>,
    pub(super) t1: SegmentParam<I>,
    pub(super) s1: Option<Segment<I>>,
    pub(super) step: SegmentParam<I>,
}

impl<I: IntNumber> Segment<I> {
    pub(super) fn is_nearly_linear(&self, sin_angle_neg_pow2: u32) -> bool {
        let chord = self.chord();
        let chord_vector = chord.vector();
        debug_assert!(chord_vector.sqr_length() != I::Wide::ZERO);

        match self {
            Segment::Line(line) => true,
            Segment::Quad(quad) => {
                let vector = quad.control_points[1] - chord.a;
                vector.sqr_length() == I::Wide::ZERO || chord_vector.is_nearly_collinear_with(vector, sin_angle_neg_pow2)
            },
            Segment::Cubic(cubic) => {
                cubic.control_points[1..2].iter().all(|&point| {
                    let vector = point - chord.a;
                    vector.sqr_length() == I::Wide::ZERO
                        || chord_vector.is_nearly_collinear_with(vector, sin_angle_neg_pow2)
                })
            },
        }
    }

    #[inline]
    pub(super) fn split(
        &self,
        t: SegmentParam<I>,
        step: SegmentParam<I>,
        a: IntPoint<I>,
        b: IntPoint<I>,
    ) -> Split<I> {
        let half_step = step.value() >> 1;
        let step = SegmentParam::new(I::from_wide(half_step));
        let t0 = SegmentParam::new(I::from_wide(t.value() - half_step));
        let t1 = SegmentParam::new(I::from_wide(t.value() + half_step));

        let [s0, s1] = self.bisect(a, b, SegmentParam::half());

        Split { t0, s0, t1, s1, step }
    }
}

#[cfg(test)]
mod tests {
    use super::Segment;
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::quad::QuadSegment;

    #[test]
    fn detects_nearly_linear_segment() {
        let segment = Segment::Cubic(CubicSegment {
            control_points: [[0, 0].into(), [4, 0].into(), [8, 1].into(), [12, 0].into()],
        });
        assert!(segment.is_nearly_linear(3));
    }

    #[test]
    fn rejects_curved_segment() {
        let segment = Segment::Quad(QuadSegment {
            control_points: [[0, 0].into(), [4, 8].into(), [8, 0].into()],
        });
        assert!(!segment.is_nearly_linear(3));
    }
}
