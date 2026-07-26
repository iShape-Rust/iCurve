use crate::int::bool::edge::CurveEdge;
use crate::int::bool::slice::{CurveMark, CurveSlice};
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::{SegmentParam, interpolate_segment_param};
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

    pub(crate) fn split(
        &mut self,
        edges: &mut Vec<CurveEdge<I>>,
        marks: &[CurveSplitMark<I>],
        slices: &mut [CurveSlice<I>],
    ) {
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
                let slice = &mut slices[edge.curve_id.0];
                Self::split_edge(edge, &marks[start..mark_index], slice, &mut self.edge_buffer);
            }
        }

        core::mem::swap(edges, &mut self.edge_buffer);
    }

    fn split_edge(
        edge: CurveEdge<I>,
        marks: &[CurveSplitMark<I>],
        slice: &mut CurveSlice<I>,
        output: &mut Vec<CurveEdge<I>>,
    ) {
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
            slice.add_mark(CurveMark {
                point: mark.point,
                param: source_param,
            });

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
    use crate::int::bool::slice::{CurveId, CurveSlice};
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::core::overlay::ShapeType;

    #[test]
    fn composes_local_params_into_source_slice_without_sorting_marks() {
        let curve = Segment::Line(LineSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(100, 0)],
        });
        let mut edges = vec![CurveEdge::full(curve, CurveId(0))];
        let mut slices = vec![CurveSlice::new(curve, ShapeType::Subject)];
        let mut splitter = CurveEdgeSplitter::new();

        splitter.split(
            &mut edges,
            &[CurveSplitMark {
                edge_index: 0,
                point: IntPoint::new(50, 0),
                param: SegmentParam::half(),
            }],
            &mut slices,
        );
        splitter.split(
            &mut edges,
            &[CurveSplitMark {
                edge_index: 1,
                point: IntPoint::new(75, 0),
                param: SegmentParam::half(),
            }],
            &mut slices,
        );

        assert_eq!(edges.len(), 3);
        assert_eq!(
            slices[0].param_at(IntPoint::new(50, 0)),
            Some(SegmentParam::from_int(1, 2))
        );
        assert_eq!(
            slices[0].param_at(IntPoint::new(75, 0)),
            Some(SegmentParam::from_int(3, 4))
        );
        assert_eq!(
            slices[0]
                .marks
                .iter()
                .map(|mark| mark.point.x)
                .collect::<Vec<_>>(),
            vec![0, 100, 50, 75]
        );
    }

    #[test]
    fn uses_edge_source_range_when_point_mark_is_ambiguous() {
        let source = Segment::Line(LineSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(100, 0)],
        });
        let edge_curve = Segment::Line(LineSegment {
            control_points: [IntPoint::new(50, 0), IntPoint::new(100, 0)],
        });
        let mut slices = vec![CurveSlice::new(source, ShapeType::Subject)];
        slices[0].add_mark(CurveMark {
            point: IntPoint::new(50, 0),
            param: SegmentParam::from_int(1, 4),
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
            &mut slices,
        );

        assert_eq!(
            slices[0].param_at(IntPoint::new(75, 0)),
            Some(SegmentParam::from_int(3, 4))
        );
        assert_eq!(edges[0].start_param, SegmentParam::half());
        assert_eq!(edges[0].end_param, SegmentParam::from_int(3, 4));
        assert_eq!(edges[1].start_param, SegmentParam::from_int(3, 4));
    }
}
