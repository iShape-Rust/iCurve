use crate::int::bool::edge::CurveEdge;
use crate::int::bool::slice::CurveSlice;
use crate::int::bool::split::{CurveEdgeSplitter, CurveSplitMark};
use crate::kernel::int::cross::intersector::{SegmentIntersectionBuffer, SegmentIntersector, SplitOptions};
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::rect::IntRect;

#[derive(Debug, Clone, Copy)]
struct CurveEdgeBounds<I: IntNumber> {
    edge_index: usize,
    rect: IntRect<I>,
}

pub(crate) struct CurvePlanarizer<I: IntNumber> {
    bounds: Vec<CurveEdgeBounds<I>>,
    active: Vec<CurveEdgeBounds<I>>,
    split_marks: Vec<CurveSplitMark<I>>,
    splitter: CurveEdgeSplitter<I>,
    intersection_buffer: SegmentIntersectionBuffer<I>,
}

impl<I: IntNumber> CurvePlanarizer<I> {
    pub(crate) fn new() -> Self {
        Self {
            bounds: Vec::new(),
            active: Vec::new(),
            split_marks: Vec::new(),
            splitter: CurveEdgeSplitter::new(),
            intersection_buffer: SegmentIntersectionBuffer::default(),
        }
    }

    pub(crate) fn planarize(
        &mut self,
        edges: &mut Vec<CurveEdge<I>>,
        slices: &mut [CurveSlice<I>],
        cross_radius: I::Wide,
    ) {
        if edges.len() < 2 {
            return;
        }

        self.build_bounds(edges);
        self.collect_split_marks(edges, cross_radius);
        self.splitter.split(edges, &self.split_marks, slices);
    }

    fn build_bounds(&mut self, edges: &[CurveEdge<I>]) {
        self.bounds.clear();
        self.bounds
            .reserve(edges.len().saturating_sub(self.bounds.capacity()));

        for (edge_index, edge) in edges.iter().enumerate() {
            let hull = edge.curve.convex_hull();
            let points = hull.as_slice();
            let first = points[0];
            let mut rect = IntRect {
                min_x: first.x,
                max_x: first.x,
                min_y: first.y,
                max_y: first.y,
            };

            for point in points.iter().skip(1) {
                rect.min_x = rect.min_x.min(point.x);
                rect.max_x = rect.max_x.max(point.x);
                rect.min_y = rect.min_y.min(point.y);
                rect.max_y = rect.max_y.max(point.y);
            }

            self.bounds.push(CurveEdgeBounds { edge_index, rect });
        }

        self.bounds
            .sort_unstable_by_key(|item| (item.rect.min_x, item.rect.min_y));
    }

    fn collect_split_marks(&mut self, edges: &[CurveEdge<I>], cross_radius: I::Wide) {
        self.active.clear();
        self.split_marks.clear();

        for bounds_index in 0..self.bounds.len() {
            let current = self.bounds[bounds_index];
            self.active.retain(|other| other.rect.max_x >= current.rect.min_x);

            for active_index in 0..self.active.len() {
                let other = self.active[active_index];
                if !other.rect.is_intersect_border_include(&current.rect) {
                    continue;
                }

                let edge_0 = edges[other.edge_index];
                let edge_1 = edges[current.edge_index];
                let intersector = SegmentIntersector::new(
                    edge_0.curve,
                    edge_1.curve,
                    SplitOptions::with_cross_radius(cross_radius),
                );
                let contacts = intersector.intersect_with_buffer(&mut self.intersection_buffer);

                for &contact in contacts {
                    CurveSplitMark::push_if_interior(
                        &mut self.split_marks,
                        other.edge_index,
                        contact.point,
                        contact.t0,
                    );
                    CurveSplitMark::push_if_interior(
                        &mut self.split_marks,
                        current.edge_index,
                        contact.point,
                        contact.t1,
                    );
                }
            }

            self.active.push(current);
        }

        CurveSplitMark::sort_and_dedup(&mut self.split_marks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::slice::{CurveId, CurveSlice};
    use crate::kernel::int::curve::chord::Chord;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::i_shape::int::IntPoint;

    fn line(id: usize, a: [i32; 2], b: [i32; 2]) -> CurveEdge<i32> {
        CurveEdge::full(
            Segment::Line(LineSegment {
                control_points: [a.into(), b.into()],
            }),
            CurveId(id),
        )
    }

    fn slices(edges: &[CurveEdge<i32>]) -> Vec<CurveSlice<i32>> {
        edges
            .iter()
            .map(|edge| CurveSlice::new(edge.curve, ShapeType::Subject))
            .collect()
    }

    #[test]
    fn splits_crossing_edges_and_preserves_curve_ids() {
        let mut edges = vec![line(0, [0, 0], [10, 10]), line(1, [0, 10], [10, 0])];
        let mut slices = slices(&edges);
        let mut planarizer = CurvePlanarizer::new();

        planarizer.planarize(&mut edges, &mut slices, 2_i64);

        assert_eq!(edges.len(), 4);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
        assert!(edges.iter().all(|edge| {
            let chord = edge.curve.chord();
            chord.a == IntPoint::new(5, 5) || chord.b == IntPoint::new(5, 5)
        }));
        assert_eq!(
            slices[0].param_at(IntPoint::new(5, 5)),
            Some(crate::kernel::int::curve::param::SegmentParam::half())
        );
        assert_eq!(
            slices[1].param_at(IntPoint::new(5, 5)),
            Some(crate::kernel::int::curve::param::SegmentParam::half())
        );
    }

    #[test]
    fn does_not_split_edges_at_shared_endpoint() {
        let mut edges = vec![line(0, [0, 0], [5, 5]), line(1, [5, 5], [10, 0])];
        let mut slices = slices(&edges);
        let mut planarizer = CurvePlanarizer::new();

        planarizer.planarize(&mut edges, &mut slices, 2_i64);

        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn collects_all_marks_before_splitting_edges() {
        let mut edges = vec![
            line(0, [0, 0], [100, 0]),
            line(1, [25, -50], [25, 50]),
            line(2, [75, -50], [75, 50]),
        ];
        let mut slices = slices(&edges);
        let mut planarizer = CurvePlanarizer::new();

        planarizer.planarize(&mut edges, &mut slices, 2_i64);

        assert_eq!(edges.len(), 7);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 3);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(2)).count(), 2);
        assert_eq!(
            slices[0].param_at(IntPoint::new(25, 0)),
            Some(crate::kernel::int::curve::param::SegmentParam::from_int(1, 4))
        );
        assert_eq!(
            slices[0].param_at(IntPoint::new(75, 0)),
            Some(crate::kernel::int::curve::param::SegmentParam::from_int(3, 4))
        );
    }

    #[test]
    fn splits_parallel_overlap_boundaries_without_changing_curve_ids() {
        let mut edges = vec![line(0, [0, 0], [64, 0]), line(1, [16, 0], [80, 0])];
        let mut slices = slices(&edges);
        let mut planarizer = CurvePlanarizer::new();

        planarizer.planarize(&mut edges, &mut slices, 2_i64);

        assert_eq!(edges.len(), 4);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
    }
}
