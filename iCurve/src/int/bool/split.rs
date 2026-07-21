use crate::int::bool::edge::CurveEdge;
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::SegmentParam;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurveSplitMark<I: IntNumber> {
    pub(crate) edge_index: usize,
    pub(crate) point: IntPoint<I>,
    pub(crate) param: SegmentParam<I>,
}

impl<I: IntNumber> CurveSplitMark<I> {
    #[inline]
    pub(crate) fn push_if_interior(
        marks: &mut Vec<Self>,
        edge_index: usize,
        point: IntPoint<I>,
        param: SegmentParam<I>,
    ) {
        let value = param.value();
        if value > I::Wide::ZERO && value < SegmentParam::<I>::DENOMINATOR {
            marks.push(Self {
                edge_index,
                point,
                param,
            });
        }
    }

    pub(crate) fn sort_and_dedup(marks: &mut Vec<Self>) {
        marks.sort_unstable_by(|a, b| {
            a.edge_index
                .cmp(&b.edge_index)
                .then_with(|| a.param.value().cmp(&b.param.value()))
                .then_with(|| a.point.cmp(&b.point))
        });
        marks.dedup();
    }
}

pub(crate) struct CurveEdgeSplitter<I: IntNumber> {
    edge_buffer: Vec<CurveEdge<I>>,
}

impl<I: IntNumber> CurveEdgeSplitter<I> {
    pub(crate) fn new() -> Self {
        Self {
            edge_buffer: Vec::new(),
        }
    }

    pub(crate) fn split(&mut self, edges: &mut Vec<CurveEdge<I>>, marks: &[CurveSplitMark<I>]) {
        if marks.is_empty() {
            return;
        }

        self.edge_buffer.clear();
        self.edge_buffer.reserve(marks.len());

        let mut mark_index = 0;
        for (edge_index, &edge) in edges.iter().enumerate() {
            let start = mark_index;
            while mark_index < marks.len() && marks[mark_index].edge_index == edge_index {
                mark_index += 1;
            }

            if start == mark_index {
                self.edge_buffer.push(edge);
            } else {
                Self::split_edge(edge, &marks[start..mark_index], &mut self.edge_buffer);
            }
        }

        core::mem::swap(edges, &mut self.edge_buffer);
    }

    fn split_edge(edge: CurveEdge<I>, marks: &[CurveSplitMark<I>], output: &mut Vec<CurveEdge<I>>) {
        let mut remaining = edge.curve;
        let mut previous = I::Wide::ZERO;

        for mark in marks {
            let value = mark.param.value();
            if value <= previous || value >= SegmentParam::<I>::DENOMINATOR {
                continue;
            }

            let numerator = value - previous;
            let denominator = SegmentParam::<I>::DENOMINATOR - previous;
            let local_param = SegmentParam::from_int(I::from_wide(numerator), I::from_wide(denominator));
            let [left, right] = remaining.split_at_point(local_param, mark.point);

            if !left.chord().is_zero_length() {
                output.push(CurveEdge {
                    curve: left,
                    curve_id: edge.curve_id,
                });
            }

            remaining = right;
            previous = value;
        }

        if !remaining.chord().is_zero_length() {
            output.push(CurveEdge {
                curve: remaining,
                curve_id: edge.curve_id,
            });
        }
    }
}
