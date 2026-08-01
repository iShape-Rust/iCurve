use crate::collections::circular_merge_list::CircularMergeList;
use crate::int::bool::data::{CurveEdgeData, CurveEdgeDataStore};
use crate::int::bool::slice::{CurveId, CurveSlice};
use crate::int::curve::path::CurvePath;
use crate::int::curve::segment::CurveSegment;
use crate::int::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;
use i_overlay::vector::edge::{DataVectorEdge, DataVectorShape};

struct CurveRun<I: IntNumber> {
    start: IntPoint<I>,
    end: IntPoint<I>,
    candidates: Vec<CurveId>,
}

impl<I: IntNumber> CurveRun<I> {
    fn try_merge(&mut self, next: &mut Self) -> bool {
        if self.end != next.start {
            debug_assert!(false, "adjacent overlay edges must share an endpoint");
            return false;
        }
        // A closed run has equal endpoint marks and cannot be represented by
        // Segment::subsegment. Keep at least two non-zero pieces around a cycle.
        if self.start == next.end {
            return false;
        }

        let candidates = intersect_sorted(&self.candidates, &next.candidates);
        if candidates.is_empty() {
            return false;
        }

        self.end = next.end;
        self.candidates = candidates;
        true
    }

    fn can_reconstruct_with(&self, curve_id: CurveId, slices: &[CurveSlice<I>]) -> bool {
        Self::can_reconstruct_range(self.start, self.end, curve_id, slices)
    }

    fn can_reconstruct_range(
        start: IntPoint<I>,
        end: IntPoint<I>,
        curve_id: CurveId,
        slices: &[CurveSlice<I>],
    ) -> bool {
        let slice = &slices[curve_id.0];
        let Some(start_param) = slice.param_at(start) else {
            return false;
        };
        let Some(end_param) = slice.param_at(end) else {
            return false;
        };

        start_param != end_param
    }

    fn is_collapsed(&self, slices: &[CurveSlice<I>]) -> bool {
        let mut has_collapsed_candidate = false;

        for &curve_id in &self.candidates {
            let slice = &slices[curve_id.0];
            let (Some(start_param), Some(end_param)) = (slice.param_at(self.start), slice.param_at(self.end))
            else {
                continue;
            };

            if start_param != end_param {
                return false;
            }
            has_collapsed_candidate = true;
        }

        has_collapsed_candidate
    }

    fn bridge_candidates(left: &Self, right: &Self, slices: &[CurveSlice<I>]) -> Vec<CurveId> {
        let mut candidates = intersect_sorted(&left.candidates, &right.candidates);
        candidates.retain(|&curve_id| {
            left.can_reconstruct_with(curve_id, slices)
                && right.can_reconstruct_with(curve_id, slices)
                && Self::can_reconstruct_range(left.start, right.end, curve_id, slices)
        });
        candidates
    }

    fn try_curve_segment(&self, slices: &[CurveSlice<I>]) -> Option<CurveSegment<I>> {
        for &curve_id in &self.candidates {
            let slice = &slices[curve_id.0];
            let Some(start_param) = slice.param_at(self.start) else {
                continue;
            };
            let Some(end_param) = slice.param_at(self.end) else {
                continue;
            };
            let Some(segment) = slice
                .curve
                .subsegment(start_param, self.start, end_param, self.end)
            else {
                continue;
            };

            return Some(CurveSegment::from_kernel_segment(segment));
        }

        None
    }

    fn into_curve_segment(self, slices: &[CurveSlice<I>]) -> CurveSegment<I> {
        if let Some(segment) = self.try_curve_segment(slices) {
            return segment;
        }

        if self.is_collapsed(slices) {
            return CurveSegment::Line { to: self.end };
        }

        let first_candidate = self.candidates.first().copied();
        let first_candidate =
            first_candidate.expect("overlay edge run must have at least one CurveId candidate");
        let first_slice = &slices[first_candidate.0];
        debug_assert!(
            false,
            "overlay edge endpoints must have marks for at least one CurveId candidate: run=({}, {})->({}, {}), first_candidate={}, start_mark={}, end_mark={}",
            self.start.x,
            self.start.y,
            self.end.x,
            self.end.y,
            first_candidate.0,
            first_slice.param_at(self.start).is_some(),
            first_slice.param_at(self.end).is_some()
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
        slices: &[CurveSlice<I>],
    ) -> Vec<CurveShape<I>> {
        let mut result = Vec::with_capacity(shapes.len());

        for shape in shapes {
            let mut contours = Vec::with_capacity(shape.len());
            for contour in shape {
                if let Some(path) = self.recompose_contour(contour, data_store, slices) {
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
        slices: &[CurveSlice<I>],
    ) -> Option<CurvePath<I>> {
        let mut runs = Vec::with_capacity(contour.len());
        for edge in contour {
            if edge.a == edge.b {
                continue;
            }

            let mut candidates = Vec::new();
            data_store.curve_ids(edge.data, &mut candidates);
            runs.push(CurveRun {
                start: edge.a,
                end: edge.b,
                candidates,
            });
        }

        let runs = self.merge_list.merge_with(runs, CurveRun::try_merge);
        let runs = self.bridge_collapsed_runs(runs, slices);
        let start = runs.first()?.start;
        let segments = runs
            .into_iter()
            .map(|run| run.into_curve_segment(slices))
            .collect();

        Some(CurvePath { start, segments })
    }

    fn bridge_collapsed_runs(
        &mut self,
        mut runs: Vec<CurveRun<I>>,
        slices: &[CurveSlice<I>],
    ) -> Vec<CurveRun<I>> {
        loop {
            let len = runs.len();
            if len <= 3 {
                // Bridging three runs would collapse the entire closed contour
                // into one run with identical endpoints.
                return runs;
            }

            let Some((collapsed_index, candidates)) = (0..len).find_map(|index| {
                if !runs[index].is_collapsed(slices) {
                    return None;
                }

                let left = &runs[(index + len - 1) % len];
                let right = &runs[(index + 1) % len];
                // The collapsed run has exhausted its own source parameter
                // range. Recover it only from a curve shared by both neighbors.
                let candidates = CurveRun::bridge_candidates(left, right, slices);
                (!candidates.is_empty()).then_some((index, candidates))
            }) else {
                return runs;
            };

            let left_index = (collapsed_index + len - 1) % len;
            runs.rotate_left(left_index);

            let merged = CurveRun {
                start: runs[0].start,
                end: runs[2].end,
                candidates,
            };
            runs[0] = merged;
            runs.drain(1..3);
            runs = self.merge_list.merge_with(runs, CurveRun::try_merge);
        }
    }
}

fn intersect_sorted(lhs: &[CurveId], rhs: &[CurveId]) -> Vec<CurveId> {
    let mut result = Vec::with_capacity(lhs.len().min(rhs.len()));
    let mut i = 0;
    let mut j = 0;

    while i < lhs.len() && j < rhs.len() {
        match lhs[i].0.cmp(&rhs[j].0) {
            core::cmp::Ordering::Less => i += 1,
            core::cmp::Ordering::Greater => j += 1,
            core::cmp::Ordering::Equal => {
                result.push(lhs[i]);
                i += 1;
                j += 1;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::slice::CurveMark;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::param::SegmentParam;
    use crate::kernel::int::curve::segment::Segment;
    use crate::kernel::int::curve::split_at::SplitAt;
    use alloc::vec;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;

    fn edge(a: IntPoint<i32>, b: IntPoint<i32>, id: usize) -> DataVectorEdge<i32, CurveEdgeData> {
        DataVectorEdge {
            a,
            b,
            fill: 0,
            data: CurveEdgeData::Single(CurveId(id)),
        }
    }

    #[test]
    fn does_not_merge_a_curve_run_into_zero_length_cycle() {
        let mut first = CurveRun {
            start: IntPoint::new(0, 0),
            end: IntPoint::new(1, 0),
            candidates: vec![CurveId(0)],
        };
        let mut closing = CurveRun {
            start: IntPoint::new(1, 0),
            end: IntPoint::new(0, 0),
            candidates: vec![CurveId(0)],
        };

        assert!(!first.try_merge(&mut closing));
    }

    #[test]
    fn ignores_zero_length_vector_edges() {
        let curve = Segment::Line(LineSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(1, 0)],
        });
        let slices = vec![CurveSlice::new(curve, ShapeType::Subject)];
        let store = CurveEdgeDataStore::default();
        let point = IntPoint::new(0, 0);
        let contour = vec![edge(point, point, 0)];

        assert!(
            CurveRecomposer::new()
                .recompose_contour(contour, &store, &slices)
                .is_none()
        );
    }

    #[test]
    fn collapsed_source_range_falls_back_to_line() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(1, 1);
        let (mut source, _) = cubic_slice();
        source.add_mark(CurveMark {
            point: p1,
            param: SegmentParam::new(0),
        });
        let closing = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [p1, p0],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![edge(p0, p1, 0), edge(p1, p0, 1)]]];

        let result =
            CurveRecomposer::new().recompose(shapes, &CurveEdgeDataStore::default(), &[source, closing]);

        assert_eq!(result[0].contours[0].segments[0], CurveSegment::Line { to: p1 });
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "overlay edge endpoints must have marks")]
    fn missing_source_mark_remains_an_error() {
        let source = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [IntPoint::new(0, 0), IntPoint::new(2, 0)],
            }),
            ShapeType::Subject,
        );
        let run = CurveRun {
            start: IntPoint::new(0, 0),
            end: IntPoint::new(1, 0),
            candidates: vec![CurveId(0)],
        };

        run.into_curve_segment(&[source]);
    }

    #[test]
    fn bridges_collapsed_run_between_compatible_curve_runs() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(2, 5);
        let p2 = IntPoint::new(6, 5);
        let p3 = IntPoint::new(8, 0);
        let (mut source, _) = cubic_slice();
        source.add_mark(CurveMark {
            point: p1,
            param: SegmentParam::from_int(1, 4),
        });
        source.add_mark(CurveMark {
            point: p2,
            param: SegmentParam::from_int(3, 4),
        });

        let closing = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [p3, p0],
            }),
            ShapeType::Subject,
        );
        let mut collapsed = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [p1, IntPoint::new(10, 10)],
            }),
            ShapeType::Subject,
        );
        collapsed.add_mark(CurveMark {
            point: p2,
            param: SegmentParam::new(0),
        });

        let shapes = vec![vec![vec![
            edge(p0, p1, 0),
            edge(p1, p2, 2),
            edge(p2, p3, 0),
            edge(p3, p0, 1),
        ]]];

        let result = CurveRecomposer::new().recompose(
            shapes,
            &CurveEdgeDataStore::default(),
            &[source, closing, collapsed],
        );

        assert_eq!(result[0].contours[0].segments.len(), 2);
        assert!(matches!(
            result[0].contours[0].segments[0],
            CurveSegment::Cubic { to, .. } if to == p3
        ));
    }

    fn cubic_slice() -> (CurveSlice<i32>, IntPoint<i32>) {
        let curve = Segment::Cubic(CubicSegment {
            control_points: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 8),
                IntPoint::new(8, 8),
                IntPoint::new(8, 0),
            ],
        });
        let middle = IntPoint::new(4, 6);
        let mut slice = CurveSlice::new(curve, ShapeType::Subject);
        slice.add_mark(CurveMark {
            point: middle,
            param: SegmentParam::half(),
        });
        (slice, middle)
    }

    fn arc_slice() -> (CurveSlice<i32>, ArcSegment<i32>, IntPoint<i32>) {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let arc = ArcSegment {
            ellipse: EllipseFrame {
                center: IntPoint::new(0, 0),
                axis_x: ArcVector { x: 100, y: 0 },
                axis_y: ArcVector { x: 0, y: 100 },
            },
            control_points: [
                IntPoint::new(100, 0),
                IntPoint::new(100, 100),
                IntPoint::new(0, 100),
            ],
            weights: [one, 759_250_125, one],
            start_phase: ArcPhase { cos: one, sin: 0 },
            end_phase: ArcPhase { cos: 0, sin: one },
            direction: ArcDirection::CounterClockwise,
        };
        let middle = arc.point_at(SegmentParam::half());
        let mut slice = CurveSlice::new(Segment::Arc(arc), ShapeType::Subject);
        slice.add_mark(CurveMark {
            point: middle,
            param: SegmentParam::half(),
        });

        (slice, arc, middle)
    }

    #[test]
    fn merges_adjacent_edges_back_into_complete_curve_slice() {
        let p0 = IntPoint::new(0, 0);
        let p3 = IntPoint::new(8, 0);
        let (cubic, middle) = cubic_slice();
        let closing = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [p3, p0],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![
            edge(p0, middle, 0),
            edge(middle, p3, 0),
            edge(p3, p0, 1),
        ]]];
        let mut recomposer = CurveRecomposer::new();

        let result = recomposer.recompose(shapes, &CurveEdgeDataStore::default(), &[cubic, closing]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].contours[0].start, p0);
        assert_eq!(result[0].contours[0].segments.len(), 2);
        match result[0].contours[0].segments[0] {
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                assert_eq!(ctrl0, IntPoint::new(0, 8));
                assert_eq!(ctrl1, IntPoint::new(8, 8));
                assert_eq!(to, p3);
            }
            _ => panic!("expected cubic segment"),
        }
    }

    #[test]
    fn restores_partial_curve_slice_in_reverse_direction() {
        let p0 = IntPoint::new(0, 0);
        let (cubic, middle) = cubic_slice();
        let closing = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [p0, middle],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![edge(middle, p0, 0), edge(p0, middle, 1)]]];
        let mut recomposer = CurveRecomposer::new();

        let result = recomposer.recompose(shapes, &CurveEdgeDataStore::default(), &[cubic, closing]);

        match result[0].contours[0].segments[0] {
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                assert_eq!(ctrl0, IntPoint::new(2, 6));
                assert_eq!(ctrl1, IntPoint::new(0, 4));
                assert_eq!(to, p0);
            }
            _ => panic!("expected cubic segment"),
        }
    }

    #[test]
    fn merges_adjacent_edges_back_into_complete_arc_slice() {
        let (arc_slice, arc, middle) = arc_slice();
        let p0 = arc.control_points[0];
        let p2 = arc.control_points[2];
        let closing = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [p2, p0],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![
            edge(p0, middle, 0),
            edge(middle, p2, 0),
            edge(p2, p0, 1),
        ]]];
        let mut recomposer = CurveRecomposer::new();

        let result = recomposer.recompose(shapes, &CurveEdgeDataStore::default(), &[arc_slice, closing]);

        assert_eq!(result[0].contours[0].segments.len(), 2);
        match result[0].contours[0].segments[0] {
            CurveSegment::Arc { arc: restored } => assert_eq!(restored, arc),
            _ => panic!("expected arc segment"),
        }
    }

    #[test]
    fn restores_partial_arc_slice_in_reverse_direction() {
        let (arc_slice, arc, middle) = arc_slice();
        let p0 = arc.control_points[0];
        let closing = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [p0, middle],
            }),
            ShapeType::Subject,
        );
        let shapes = vec![vec![vec![edge(middle, p0, 0), edge(p0, middle, 1)]]];
        let mut recomposer = CurveRecomposer::new();
        let expected = arc.split_at_left(SegmentParam::half()).reversed();

        let result = recomposer.recompose(shapes, &CurveEdgeDataStore::default(), &[arc_slice, closing]);

        match result[0].contours[0].segments[0] {
            CurveSegment::Arc { arc: restored } => assert_eq!(restored, expected),
            _ => panic!("expected arc segment"),
        }
    }
}
