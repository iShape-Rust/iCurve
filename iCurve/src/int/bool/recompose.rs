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

        let candidates = intersect_sorted(&self.candidates, &next.candidates);
        if candidates.is_empty() {
            return false;
        }

        self.end = next.end;
        self.candidates = candidates;
        true
    }

    fn into_curve_segment(self, slices: &[CurveSlice<I>]) -> CurveSegment<I> {
        for curve_id in self.candidates {
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

            return CurveSegment::from_kernel_segment(segment);
        }

        debug_assert!(
            false,
            "overlay edge endpoints must have marks for at least one CurveId candidate"
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
            let mut candidates = Vec::new();
            data_store.curve_ids(edge.data, &mut candidates);
            runs.push(CurveRun {
                start: edge.a,
                end: edge.b,
                candidates,
            });
        }

        let runs = self.merge_list.merge_with(runs, CurveRun::try_merge);
        let start = runs.first()?.start;
        let segments = runs
            .into_iter()
            .map(|run| run.into_curve_segment(slices))
            .collect();

        Some(CurvePath { start, segments })
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
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::param::SegmentParam;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::core::overlay::ShapeType;

    fn edge(a: IntPoint<i32>, b: IntPoint<i32>, id: usize) -> DataVectorEdge<i32, CurveEdgeData> {
        DataVectorEdge {
            a,
            b,
            fill: 0,
            data: CurveEdgeData::Single(CurveId(id)),
        }
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
}
