use crate::bool::overlay::CurveOverlay;
use crate::flatten::approx::{LineApproximation, LineApproximationSplit};
use crate::bool::segment::SegmentRange;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;
use crate::kernel::curve::split_at::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(crate) fn simplify_segments(&self) -> Vec<SegmentRange<P::Scalar>> {
        let mut ranges = Vec::<SegmentRange<P::Scalar>>::with_capacity(16 * self.segments.len());

        let min_segment_length = self.adapter.len_to_float(self.options.split.min_length);
        let min_segment_sqr_length = min_segment_length * min_segment_length;

        let min_cos = self.options.split.max_angle.cos();
        let line_approximation = LineApproximation {
            min_cos,
            min_segment_sqr_length,
        };

        for (i, s) in self.segments.iter().enumerate() {
            match &s.segment {
                Segment::Line(_) => {
                    ranges.push(SegmentRange::full(i));
                }
                Segment::Quad(quad) => {
                    quad.split_range(
                        i,
                        P::Scalar::from_float(0.0),
                        P::Scalar::from_float(1.0),
                        line_approximation,
                        &mut ranges,
                    );
                }
                Segment::Cubic(cubic) => {
                    cubic.split_range(
                        i,
                        P::Scalar::from_float(0.0),
                        P::Scalar::from_float(1.0),
                        line_approximation,
                        &mut ranges,
                    );
                }
            }
        }

        ranges
    }
}

impl<T: FloatNumber> QuadSegment<T> {
    fn split_range(
        &self,
        index: usize,
        t0: T,
        t1: T,
        line_approximation: LineApproximation<T>,
        output: &mut Vec<SegmentRange<T>>,
    ) {
        if !self.is_split_required(line_approximation) {
            output.push(SegmentRange::new(index, t0, t1));
            return;
        }

        let tm = (t0 + t1) * T::HALF;
        let [lt, rt] = self.split_at(T::HALF);
        lt.split_range(index, t0, tm, line_approximation, output);
        rt.split_range(index, tm, t1, line_approximation, output);
    }
}

impl<T: FloatNumber> CubicSegment<T> {
    fn split_range(
        &self,
        index: usize,
        t0: T,
        t1: T,
        line_approximation: LineApproximation<T>,
        output: &mut Vec<SegmentRange<T>>,
    ) {
        if !self.is_split_required(line_approximation) {
            output.push(SegmentRange::new(index, t0, t1));
            return;
        }

        let tm = (t0 + t1) * T::HALF;
        let [lt, rt] = self.split_at(T::HALF);
        lt.split_range(index, t0, tm, line_approximation, output);
        rt.split_range(index, tm, t1, line_approximation, output);
    }
}
