mod line_line;
mod quad_quad;

use crate::flatten::segment::{NormalizedSegment, SegmentRange};
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::point::IntPoint;

#[derive(Clone, Copy)]
pub(super) struct CurveSpan<'a, P: FloatPointCompatible, I: IntNumber> {
    pub(super) start: IntPoint<I>,
    pub(super) end: IntPoint<I>,
    pub(super) segment: &'a NormalizedSegment<P>,
    pub(super) range: SegmentRange<P::Scalar>,
}

impl<'a, P, I> CurveSpan<'a, P, I>
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    #[inline(always)]
    pub(super) fn new(
        start: IntPoint<I>,
        end: IntPoint<I>,
        segment: &'a NormalizedSegment<P>,
        range: SegmentRange<P::Scalar>,
    ) -> Self {
        Self {
            start,
            end,
            segment,
            range,
        }
    }

    pub(super) fn can_recombine_with(self, next: Self, adapter: &FloatPointAdapter<P, I>) -> bool {
        debug_assert!(self.end == next.start);

        match (self.segment, next.segment) {
            (NormalizedSegment::Line(_), NormalizedSegment::Line(_)) => {
                line_line::can_recombine(self, next)
            }
            (NormalizedSegment::Quad(a), NormalizedSegment::Quad(b)) => {
                quad_quad::can_recombine(self, next, a, b, adapter)
            }
            _ => false,
        }
    }
}

fn range_split_parameter<F: FloatNumber>(prev: SegmentRange<F>, next: SegmentRange<F>) -> Option<F> {
    let t0 = prev.t0.value();
    let t1 = prev.t1.value();
    let t2 = next.t1.value();
    let denom = t2 - t0;

    if denom == F::from_float(0.0) {
        None
    } else {
        Some((t1 - t0) / denom)
    }
}

fn close_parameter<F: FloatNumber>(a: F, b: F) -> bool {
    (a - b).abs() <= F::from_float(0.0001)
}

fn point_at<P: FloatPointCompatible>(a: P, b: P, t: P::Scalar) -> P {
    P::from_xy(a.x() + (b.x() - a.x()) * t, a.y() + (b.y() - a.y()) * t)
}

fn close_point<P, I>(a: P, b: P, adapter: &FloatPointAdapter<P, I>) -> bool
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    let sqr_distance = dx * dx + dy * dy;

    adapter.round_sqr_len_to_int(sqr_distance) <= I::Wide::ONE
}
