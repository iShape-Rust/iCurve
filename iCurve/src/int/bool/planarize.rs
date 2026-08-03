use crate::int::CurveInt;
use crate::int::bool::bounds::CurveBoundsBuffer;
use crate::int::bool::edge::CurveEdge;
use crate::int::bool::split::{CurveEdgeSplitter, CurveSplitMark};
use crate::kernel::int::cross::intersector::{SegmentIntersectionBuffer, SegmentIntersector, SplitOptions};
use crate::kernel::int::curve::segment::Segment;
use alloc::vec::Vec;
use core::cmp::Ordering;

pub(crate) struct CurvePlanarizer<I: CurveInt> {
    split_marks: Vec<CurveSplitMark<I>>,
    split_marks_buffer: Vec<CurveSplitMark<I>>,
    splitter: CurveEdgeSplitter<I>,
    intersection_buffer: SegmentIntersectionBuffer<I>,
}

impl<I: CurveInt + i_key_sort::sort::key::SortKey> CurvePlanarizer<I> {
    pub(crate) fn new() -> Self {
        Self {
            split_marks: Vec::new(),
            split_marks_buffer: Vec::new(),
            splitter: CurveEdgeSplitter::new(),
            intersection_buffer: SegmentIntersectionBuffer::default(),
        }
    }

    pub(crate) fn planarize(
        &mut self,
        edges: &mut Vec<CurveEdge<I>>,
        cross_radius: I::Wide,
        bounds: &mut CurveBoundsBuffer<I>,
    ) {
        if edges.len() < 2 {
            return;
        }

        bounds.build(edges);
        self.collect_split_marks(edges, cross_radius, bounds);
        self.splitter.split(edges, &self.split_marks);
    }

    fn collect_split_marks(
        &mut self,
        edges: &[CurveEdge<I>],
        cross_radius: I::Wide,
        bounds: &mut CurveBoundsBuffer<I>,
    ) {
        bounds.active.clear();
        self.split_marks.clear();

        for bounds_index in 0..bounds.bounds.len() {
            let current = bounds.bounds[bounds_index];
            bounds
                .active
                .retain(|other| other.rect.max_x >= current.rect.min_x);

            for active_index in 0..bounds.active.len() {
                let other = bounds.active[active_index];
                if !other.rect.is_intersect_border_include(&current.rect) {
                    continue;
                }

                let edge_0 = edges[other.edge_index];
                let edge_1 = edges[current.edge_index];
                let (first_index, first_edge, second_index, second_edge) =
                    if Self::compare_geometry(&edge_0.curve, &edge_1.curve) != Ordering::Greater {
                        (other.edge_index, edge_0, current.edge_index, edge_1)
                    } else {
                        (current.edge_index, edge_1, other.edge_index, edge_0)
                    };
                let intersector = SegmentIntersector::new(
                    first_edge.curve,
                    second_edge.curve,
                    SplitOptions::with_cross_radius(cross_radius),
                );
                let contacts = intersector.intersect_with_buffer(&mut self.intersection_buffer);

                for &contact in contacts {
                    CurveSplitMark::push_if_interior(
                        &mut self.split_marks,
                        first_index,
                        contact.point,
                        contact.t0,
                    );
                    CurveSplitMark::push_if_interior(
                        &mut self.split_marks,
                        second_index,
                        contact.point,
                        contact.t1,
                    );
                }
            }

            bounds.active.push(current);
        }

        CurveSplitMark::sort_and_dedup(&mut self.split_marks, &mut self.split_marks_buffer);
    }

    fn compare_geometry(lhs: &Segment<I>, rhs: &Segment<I>) -> Ordering {
        Self::segment_rank(lhs)
            .cmp(&Self::segment_rank(rhs))
            .then_with(|| Self::control_points(lhs).cmp(Self::control_points(rhs)))
            .then_with(|| match (lhs, rhs) {
                (Segment::Arc(lhs), Segment::Arc(rhs)) => lhs.weights.cmp(&rhs.weights),
                _ => Ordering::Equal,
            })
    }

    #[inline]
    fn control_points(segment: &Segment<I>) -> &[i_overlay::i_shape::int::IntPoint<I>] {
        match segment {
            Segment::Line(line) => &line.control_points,
            Segment::Quad(quad) => &quad.control_points,
            Segment::Cubic(cubic) => &cubic.control_points,
            Segment::Arc(arc) => &arc.control_points,
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::source::CurveId;
    use crate::kernel::int::curve::chord::Chord;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::i_shape::int::IntPoint;

    fn line(id: usize, a: [i32; 2], b: [i32; 2]) -> CurveEdge<i32> {
        CurveEdge::full(
            Segment::Line(LineSegment {
                control_points: [a.into(), b.into()],
            }),
            CurveId(id),
        )
    }

    #[test]
    fn splits_crossing_edges_and_preserves_curve_ids() {
        let mut edges = vec![line(0, [0, 0], [10, 10]), line(1, [0, 10], [10, 0])];
        let mut planarizer = CurvePlanarizer::new();
        let mut bounds = CurveBoundsBuffer::new();

        planarizer.planarize(&mut edges, 2_i64, &mut bounds);

        assert_eq!(edges.len(), 4);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
        assert!(edges.iter().all(|edge| {
            let chord = edge.curve.chord();
            chord.a == IntPoint::new(5, 5) || chord.b == IntPoint::new(5, 5)
        }));
        assert_eq!(
            edges[0].end_param,
            crate::kernel::int::curve::param::SegmentParam::half()
        );
    }

    #[test]
    fn does_not_split_edges_at_shared_endpoint() {
        let mut edges = vec![line(0, [0, 0], [5, 5]), line(1, [5, 5], [10, 0])];
        let mut planarizer = CurvePlanarizer::new();
        let mut bounds = CurveBoundsBuffer::new();

        planarizer.planarize(&mut edges, 2_i64, &mut bounds);

        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn collects_all_marks_before_splitting_edges() {
        let mut edges = vec![
            line(0, [0, 0], [100, 0]),
            line(1, [25, -50], [25, 50]),
            line(2, [75, -50], [75, 50]),
        ];
        let mut planarizer = CurvePlanarizer::new();
        let mut bounds = CurveBoundsBuffer::new();

        planarizer.planarize(&mut edges, 2_i64, &mut bounds);

        assert_eq!(edges.len(), 7);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 3);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(2)).count(), 2);
        assert_eq!(
            edges[0].end_param,
            crate::kernel::int::curve::param::SegmentParam::from_int(1, 4)
        );
        assert_eq!(
            edges[1].end_param,
            crate::kernel::int::curve::param::SegmentParam::from_int(3, 4)
        );
    }

    #[test]
    fn splits_parallel_overlap_boundaries_without_changing_curve_ids() {
        let mut edges = vec![line(0, [0, 0], [64, 0]), line(1, [16, 0], [80, 0])];
        let mut planarizer = CurvePlanarizer::new();
        let mut bounds = CurveBoundsBuffer::new();

        planarizer.planarize(&mut edges, 2_i64, &mut bounds);

        assert_eq!(edges.len(), 4);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
    }
}
