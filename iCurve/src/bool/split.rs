use crate::bool::overlay::CurveOverlay;
use crate::flatten::approx::{LineApproximation, LineApproximationSplit};
use crate::flatten::segment::{ArcSegment, CubicSegment, NormalizedSegment, QuadSegment, SegmentRange};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(super) fn split(&self) -> Vec<SegmentRange<P::Scalar>> {
        let ranges = self.pre_split();
        ranges
    }
    fn pre_split(&self) -> Vec<SegmentRange<P::Scalar>> {
        let mut ranges = Vec::<SegmentRange<P::Scalar>>::with_capacity(16 * self.segments.len());

        let min_segment_length = self.adapter.len_to_float(self.options.split.min_length);
        let min_segment_sqr_length = min_segment_length * min_segment_length;

        let min_cos = self.options.split.max_angle.cos();
        let line_approximation = LineApproximation {
            min_cos,
            min_segment_sqr_length,
        };

        for (i, s) in self.segments.iter().enumerate() {
            match &s.normalized_segment {
                NormalizedSegment::Line(_) => {
                    ranges.push(SegmentRange::full(i));
                }
                NormalizedSegment::Quad(quad) => {
                    quad.split_range(
                        i,
                        P::Scalar::from_float(0.0),
                        P::Scalar::from_float(1.0),
                        line_approximation,
                        &mut ranges,
                    );
                }
                NormalizedSegment::Cubic(cubic) => {
                    cubic.split_range(
                        i,
                        P::Scalar::from_float(0.0),
                        P::Scalar::from_float(1.0),
                        line_approximation,
                        &mut ranges,
                    );
                }
                NormalizedSegment::Arc(arc) => {
                    arc.split_range(
                        i,
                        self.options.split.max_angle,
                        min_segment_sqr_length,
                        &mut ranges,
                    );
                }
            }
        }

        ranges
    }
}

impl<P: FloatPointCompatible> QuadSegment<P> {
    fn split_range(
        &self,
        index: usize,
        t0: P::Scalar,
        t1: P::Scalar,
        line_approximation: LineApproximation<P::Scalar>,
        output: &mut Vec<SegmentRange<P::Scalar>>,
    ) {
        if !self.is_split_required(line_approximation) {
            output.push(SegmentRange::new(index, t0, t1));
            return;
        }

        let tm = (t0 + t1) * P::Scalar::from_float(0.5);
        let [lt, rt] = self.split_at_half();
        lt.split_range(index, t0, tm, line_approximation, output);
        rt.split_range(index, tm, t1, line_approximation, output);
    }
}

impl<P: FloatPointCompatible> CubicSegment<P> {
    fn split_range(
        &self,
        index: usize,
        t0: P::Scalar,
        t1: P::Scalar,
        line_approximation: LineApproximation<P::Scalar>,
        output: &mut Vec<SegmentRange<P::Scalar>>,
    ) {
        if !self.is_split_required(line_approximation) {
            output.push(SegmentRange::new(index, t0, t1));
            return;
        }

        let tm = (t0 + t1) * P::Scalar::from_float(0.5);
        let [lt, rt] = self.split_at_half();
        lt.split_range(index, t0, tm, line_approximation, output);
        rt.split_range(index, tm, t1, line_approximation, output);
    }
}

impl<P: FloatPointCompatible> ArcSegment<P> {
    fn split_range(
        &self,
        index: usize,
        max_angle: P::Scalar,
        min_segment_sqr_length: P::Scalar,
        output: &mut Vec<SegmentRange<P::Scalar>>,
    ) {
        let zero = P::Scalar::from_float(0.0);
        let one = P::Scalar::from_float(1.0);
        let sweep_angle = self.sweep_angle.abs();
        let radius = self.radii.x().abs().max(self.radii.y().abs());
        let max_arc_length = radius * sweep_angle;

        if max_angle <= zero
            || sweep_angle <= max_angle
            || max_arc_length * max_arc_length <= min_segment_sqr_length
        {
            output.push(SegmentRange::full(index));
            return;
        }

        let ratio = sweep_angle / max_angle;
        let mut count = ratio.to_usize();
        if P::Scalar::from_usize(count) < ratio {
            count += 1;
        }
        let count = count.max(1);
        let step = one / P::Scalar::from_usize(count);
        let mut t0 = zero;

        for i in 1..=count {
            let t1 = if i == count {
                one
            } else {
                P::Scalar::from_usize(i) * step
            };
            output.push(SegmentRange::new(index, t0, t1));
            t0 = t1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::segment::{ArcSegment, SegmentParam};

    #[test]
    fn arc_split_uses_sweep_angle_count() {
        let arc = ArcSegment {
            p0: [1.0, 0.0],
            p1: [0.0, 1.0],
            center: [0.0, 0.0],
            radii: [1.0, 1.0],
            rotation: 0.0,
            start_angle: 0.0,
            sweep_angle: core::f64::consts::FRAC_PI_2,
        };
        let mut output = Vec::new();

        arc.split_range(7, core::f64::consts::FRAC_PI_8, 0.0, &mut output);

        assert_eq!(output.len(), 4);
        assert_eq!(output[0].segment_index, 7);
        assert_eq!(output[0].t0, SegmentParam::Start);
        assert_eq!(output[0].t1, SegmentParam::Inner(0.25));
        assert_eq!(output[3].t0, SegmentParam::Inner(0.75));
        assert_eq!(output[3].t1, SegmentParam::End);
    }
}
