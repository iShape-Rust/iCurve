use crate::collections::circular_merge_list::CircularMergeList;
use crate::int::bool::data::{CurveEdgeData, CurveEdgeDataStore, CurveSourceSpan};
use crate::int::bool::source::CurveSource;
use crate::int::curve::path::CurvePath;
use crate::int::curve::segment::CurveSegment;
use crate::int::curve::shape::CurveShape;
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::segment::Segment;
use alloc::vec::Vec;
use core::cmp::Ordering;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;
use i_overlay::vector::edge::{DataVectorEdge, DataVectorShape};

struct CurveRun<I: IntNumber> {
    start: IntPoint<I>,
    end: IntPoint<I>,
    candidates: Vec<CurveSourceSpan>,
}

impl<I: IntNumber> CurveRun<I> {
    fn try_merge(&mut self, next: &mut Self) -> bool {
        if self.end != next.start {
            debug_assert!(false, "adjacent overlay edges must share an endpoint");
            return false;
        }
        // A closed run cannot be represented by one source subsegment.
        if self.start == next.end {
            return false;
        }

        let candidates = Self::joined_candidates(&self.candidates, &next.candidates);
        if candidates.is_empty() {
            return false;
        }

        self.end = next.end;
        self.candidates = candidates;
        true
    }

    fn joined_candidates(lhs: &[CurveSourceSpan], rhs: &[CurveSourceSpan]) -> Vec<CurveSourceSpan> {
        let mut result = Vec::new();

        for &left in lhs {
            if left.is_collapsed() {
                continue;
            }
            for &right in rhs {
                if right.is_collapsed() || left.curve_id != right.curve_id || left.end != right.start {
                    continue;
                }
                result.push(CurveSourceSpan {
                    curve_id: left.curve_id,
                    start: left.start,
                    end: right.end,
                });
            }
        }

        result.sort_unstable();
        result.dedup();
        result
    }

    fn is_collapsed(&self) -> bool {
        !self.candidates.is_empty() && self.candidates.iter().all(|span| span.is_collapsed())
    }

    fn bridge_candidates(left: &Self, right: &Self) -> Vec<CurveSourceSpan> {
        Self::joined_candidates(&left.candidates, &right.candidates)
    }

    fn try_curve_segment(&self, sources: &[CurveSource<I>]) -> Option<CurveSegment<I>> {
        let mut best: Option<Segment<I>> = None;

        for &span in &self.candidates {
            if span.is_collapsed() {
                continue;
            }
            let source = &sources[span.curve_id.0];
            let Some(segment) = source.curve.subsegment(
                span.start.to_segment(),
                self.start,
                span.end.to_segment(),
                self.end,
            ) else {
                continue;
            };

            if best
                .as_ref()
                .is_none_or(|current| Self::compare_segments(&segment, current) == Ordering::Less)
            {
                best = Some(segment);
            }
        }

        best.map(CurveSegment::from_kernel_segment)
    }

    fn compare_segments(lhs: &Segment<I>, rhs: &Segment<I>) -> Ordering {
        Self::chord_deviation(lhs)
            .cmp(&Self::chord_deviation(rhs))
            .then_with(|| {
                let quarter = SegmentParam::from_int(I::ONE, I::FOUR);
                lhs.point_at(quarter).cmp(&rhs.point_at(quarter))
            })
            .then_with(|| {
                lhs.point_at(SegmentParam::half())
                    .cmp(&rhs.point_at(SegmentParam::half()))
            })
            .then_with(|| {
                let three_quarters = SegmentParam::from_int(I::from_u32(3), I::FOUR);
                lhs.point_at(three_quarters).cmp(&rhs.point_at(three_quarters))
            })
            .then_with(|| Self::segment_rank(lhs).cmp(&Self::segment_rank(rhs)))
    }

    fn chord_deviation(segment: &Segment<I>) -> I::WideUInt {
        let chord = segment.chord();
        let vector = chord.vector();
        let mut deviation = I::WideUInt::ZERO;

        let controls: &[IntPoint<I>] = match segment {
            Segment::Line(line) => &line.control_points,
            Segment::Quad(quad) => &quad.control_points,
            Segment::Cubic(cubic) => &cubic.control_points,
            Segment::Arc(arc) => &arc.control_points,
        };
        for &point in controls.iter().skip(1).take(controls.len().saturating_sub(2)) {
            deviation = deviation.max(vector.cross_product(point - chord.a).unsigned_abs());
        }

        deviation
    }

    #[inline]
    fn segment_rank(segment: &Segment<I>) -> u8 {
        match segment {
            Segment::Line(_) => 0,
            Segment::Quad(_) => 1,
            Segment::Cubic(_) => 2,
            Segment::Arc(_) => 3,
        }
    }

    fn into_curve_segment(self, sources: &[CurveSource<I>]) -> CurveSegment<I> {
        if let Some(segment) = self.try_curve_segment(sources) {
            return segment;
        }

        debug_assert!(
            self.is_collapsed(),
            "a non-collapsed overlay edge must retain a reconstructable source span"
        );
        CurveSegment::Line { to: self.end }
    }
}

pub(crate) struct CurveRecomposer<I: IntNumber> {
    merge_list: CircularMergeList<CurveRun<I>>,
}

impl<I: IntNumber> CurveRecomposer<I> {
    pub(crate) fn new() -> Self {
        Self {
            merge_list: CircularMergeList::with_capacity(0),
        }
    }

    pub(crate) fn recompose(
        &mut self,
        shapes: Vec<DataVectorShape<I, CurveEdgeData>>,
        data_store: &CurveEdgeDataStore,
        sources: &[CurveSource<I>],
    ) -> Vec<CurveShape<I>> {
        let mut result = Vec::with_capacity(shapes.len());

        for shape in shapes {
            let mut contours = Vec::with_capacity(shape.len());
            for contour in shape {
                if let Some(path) = self.recompose_contour(contour, data_store, sources) {
                    contours.push(path);
                }
            }

            if !contours.is_empty() {
                result.push(CurveShape { contours });
            }
        }

        result
    }

    fn recompose_contour(
        &mut self,
        contour: Vec<DataVectorEdge<I, CurveEdgeData>>,
        data_store: &CurveEdgeDataStore,
        sources: &[CurveSource<I>],
    ) -> Option<CurvePath<I>> {
        let mut runs = Vec::with_capacity(contour.len());
        for edge in contour {
            if edge.a == edge.b {
                continue;
            }

            let mut candidates = Vec::new();
            data_store.spans(edge.data, &mut candidates);
            runs.push(CurveRun {
                start: edge.a,
                end: edge.b,
                candidates,
            });
        }

        let runs = self.merge_list.merge_with(runs, CurveRun::try_merge);
        let runs = self.bridge_collapsed_runs(runs);
        let start = runs.first()?.start;
        let segments = runs
            .into_iter()
            .map(|run| run.into_curve_segment(sources))
            .collect();

        Some(CurvePath { start, segments })
    }

    fn bridge_collapsed_runs(&mut self, mut runs: Vec<CurveRun<I>>) -> Vec<CurveRun<I>> {
        loop {
            let len = runs.len();
            if len <= 3 {
                // Bridging three runs would collapse the complete closed contour.
                return runs;
            }

            let Some((collapsed_index, candidates)) = (0..len).find_map(|index| {
                if !runs[index].is_collapsed() {
                    return None;
                }

                let left = &runs[(index + len - 1) % len];
                let right = &runs[(index + 1) % len];
                let candidates = CurveRun::bridge_candidates(left, right);
                (!candidates.is_empty()).then_some((index, candidates))
            }) else {
                return runs;
            };

            let left_index = (collapsed_index + len - 1) % len;
            runs.rotate_left(left_index);

            runs[0] = CurveRun {
                start: runs[0].start,
                end: runs[2].end,
                candidates,
            };
            runs.drain(1..3);
            runs = self.merge_list.merge_with(runs, CurveRun::try_merge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::data::CurveParam;
    use crate::int::bool::source::CurveId;
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::param::SegmentParam;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::core::overlay::ShapeType;

    fn span(id: usize, start: (i32, i32), end: (i32, i32)) -> CurveSourceSpan {
        CurveSourceSpan {
            curve_id: CurveId(id),
            start: CurveParam::from_segment(SegmentParam::<i32>::from_int(start.0, start.1)),
            end: CurveParam::from_segment(SegmentParam::<i32>::from_int(end.0, end.1)),
        }
    }

    fn edge(a: IntPoint<i32>, b: IntPoint<i32>, span: CurveSourceSpan) -> DataVectorEdge<i32, CurveEdgeData> {
        DataVectorEdge {
            a,
            b,
            fill: 0,
            data: CurveEdgeData::Single(span),
        }
    }

    fn cubic_source() -> CurveSource<i32> {
        CurveSource::new(
            Segment::Cubic(CubicSegment {
                control_points: [
                    IntPoint::new(0, 0),
                    IntPoint::new(2, 6),
                    IntPoint::new(6, 6),
                    IntPoint::new(8, 0),
                ],
            }),
            ShapeType::Subject,
        )
    }

    #[test]
    fn merges_adjacent_source_spans_into_one_curve() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(4, 5);
        let p2 = IntPoint::new(8, 0);
        let closing = CurveSource::new(
            Segment::Line(LineSegment {
                control_points: [p2, p0],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![
            edge(p0, p1, span(0, (0, 1), (1, 2))),
            edge(p1, p2, span(0, (1, 2), (1, 1))),
            edge(p2, p0, span(1, (0, 1), (1, 1))),
        ]]];

        let result = CurveRecomposer::new().recompose(
            shapes,
            &CurveEdgeDataStore::default(),
            &[cubic_source(), closing],
        );

        assert_eq!(result[0].contours[0].segments.len(), 2);
        assert!(matches!(result[0].contours[0].segments[0], CurveSegment::Cubic { to, .. } if to == p2));
    }

    #[test]
    fn collapsed_source_span_falls_back_to_line() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 1);
        let closing = CurveSource::new(
            Segment::Line(LineSegment {
                control_points: [p1, p0],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![
            edge(p0, p1, span(0, (1, 2), (1, 2))),
            edge(p1, p0, span(1, (0, 1), (1, 1))),
        ]]];

        let result = CurveRecomposer::new().recompose(
            shapes,
            &CurveEdgeDataStore::default(),
            &[cubic_source(), closing],
        );

        assert_eq!(result[0].contours[0].segments[0], CurveSegment::Line { to: p1 });
    }

    #[test]
    fn bridges_collapsed_span_between_compatible_neighbors() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(2, 5);
        let p2 = IntPoint::new(6, 5);
        let p3 = IntPoint::new(8, 0);
        let closing = CurveSource::new(
            Segment::Line(LineSegment {
                control_points: [p3, p0],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![
            edge(p0, p1, span(0, (0, 1), (1, 4))),
            edge(p1, p2, span(0, (1, 4), (1, 4))),
            edge(p2, p3, span(0, (1, 4), (1, 1))),
            edge(p3, p0, span(1, (0, 1), (1, 1))),
        ]]];

        let result = CurveRecomposer::new().recompose(
            shapes,
            &CurveEdgeDataStore::default(),
            &[cubic_source(), closing],
        );

        assert_eq!(result[0].contours[0].segments.len(), 2);
        assert!(matches!(result[0].contours[0].segments[0], CurveSegment::Cubic { to, .. } if to == p3));
    }
}
