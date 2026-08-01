use crate::int::CurveInt;
use crate::int::bool::edge::CurveEdge;
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::{SegmentParam, interpolate_segment_param};
use alloc::vec::Vec;
use i_key_sort::sort::one_key_cmp::OneKeyAndCmpSort;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurveSplitMark<I: CurveInt> {
    pub(crate) edge_index: usize,
    pub(crate) point: IntPoint<I>,
    pub(crate) param: SegmentParam<I>,
}

impl<I: CurveInt> CurveSplitMark<I> {
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

    pub(crate) fn sort_and_dedup(marks: &mut Vec<Self>, buffer: &mut Vec<Self>) {
        marks.sort_by_one_key_then_by_and_buffer(
            false,
            buffer,
            |m| m.edge_index,
            |m0, m1| {
                m0.param
                    .value()
                    .cmp(&m1.param.value())
                    .then_with(|| m0.point.cmp(&m1.point))
            },
        );
        marks.dedup();
    }
}

pub(crate) struct CurveEdgeSplitter<I: CurveInt> {
    edge_buffer: Vec<CurveEdge<I>>,
}

impl<I: CurveInt> CurveEdgeSplitter<I> {
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
        let source_start = edge.start_param;
        let source_end = edge.end_param;

        let mut remaining = edge.curve;
        let mut previous = I::Wide::ZERO;
        let mut remaining_start = source_start;

        for mark in marks {
            let value = mark.param.value();
            if value <= previous || value >= SegmentParam::<I>::DENOMINATOR {
                continue;
            }

            let numerator = value - previous;
            let denominator = SegmentParam::<I>::DENOMINATOR - previous;
            let local_param = SegmentParam::from_int(I::from_wide(numerator), I::from_wide(denominator));
            let [left, right] = remaining.split_at_point(local_param, mark.point);
            let source_param = interpolate_segment_param(source_start, source_end, mark.param);

            if !left.chord().is_zero_length() {
                output.push(CurveEdge::new(left, edge.curve_id, remaining_start, source_param));
            }

            remaining = right;
            previous = value;
            remaining_start = source_param;
        }

        if !remaining.chord().is_zero_length() {
            output.push(CurveEdge::new(
                remaining,
                edge.curve_id,
                remaining_start,
                source_end,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::source::CurveId;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;

    #[test]
    fn composes_local_params_into_source_edges() {
        let curve = Segment::Line(LineSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(100, 0)],
        });
        let mut edges = vec![CurveEdge::full(curve, CurveId(0))];
        let mut splitter = CurveEdgeSplitter::new();

        splitter.split(
            &mut edges,
            &[CurveSplitMark {
                edge_index: 0,
                point: IntPoint::new(50, 0),
                param: SegmentParam::half(),
            }],
        );
        splitter.split(
            &mut edges,
            &[CurveSplitMark {
                edge_index: 1,
                point: IntPoint::new(75, 0),
                param: SegmentParam::half(),
            }],
        );

        assert_eq!(edges.len(), 3);
        assert_eq!(edges[0].end_param, SegmentParam::from_int(1, 2));
        assert_eq!(edges[1].start_param, SegmentParam::from_int(1, 2));
        assert_eq!(edges[1].end_param, SegmentParam::from_int(3, 4));
        assert_eq!(edges[2].start_param, SegmentParam::from_int(3, 4));
    }

    #[test]
    fn uses_edge_source_range_when_point_mark_is_ambiguous() {
        let edge_curve = Segment::Line(LineSegment {
            control_points: [IntPoint::new(50, 0), IntPoint::new(100, 0)],
        });
        let mut edges = vec![CurveEdge::new(
            edge_curve,
            CurveId(0),
            SegmentParam::half(),
            SegmentParam::new(SegmentParam::<i32>::DENOMINATOR as i32),
        )];
        let mut splitter = CurveEdgeSplitter::new();

        splitter.split(
            &mut edges,
            &[CurveSplitMark {
                edge_index: 0,
                point: IntPoint::new(75, 0),
                param: SegmentParam::half(),
            }],
        );

        assert_eq!(edges[0].start_param, SegmentParam::half());
        assert_eq!(edges[0].end_param, SegmentParam::from_int(3, 4));
        assert_eq!(edges[1].start_param, SegmentParam::from_int(3, 4));
    }
}
