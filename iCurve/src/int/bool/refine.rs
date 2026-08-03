use crate::int::CurveInt;
use crate::int::bool::bounds::CurveBoundsBuffer;
use crate::int::bool::edge::CurveEdge;
use crate::int::bool::split::{CurveEdgeSplitter, CurveSplitMark};
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

pub(crate) struct CurveContainmentRefiner<I: CurveInt> {
    targets: Vec<bool>,
    split_marks: Vec<CurveSplitMark<I>>,
    splitter: CurveEdgeSplitter<I>,
}

impl<I: CurveInt> CurveContainmentRefiner<I> {
    pub(crate) fn new() -> Self {
        Self {
            targets: Vec::new(),
            split_marks: Vec::new(),
            splitter: CurveEdgeSplitter::new(),
        }
    }

    pub(crate) fn refine(
        &mut self,
        edges: &mut Vec<CurveEdge<I>>,
        subdivision_power: u32,
        max_iterations: u32,
        min_chord_length_power: u32,
        angle_tolerance_power: u32,
        bounds: &mut CurveBoundsBuffer<I>,
    ) {
        if edges.len() < 2 || subdivision_power == 0 || max_iterations == 0 {
            return;
        }

        let subdivision_count = 1_u32 << subdivision_power;
        for _ in 0..max_iterations {
            if !self.collect_targets(edges, angle_tolerance_power, bounds) {
                break;
            }

            self.build_split_marks(edges, subdivision_count, min_chord_length_power);
            if self.split_marks.is_empty() {
                break;
            }
            self.splitter.split(edges, &self.split_marks);
        }
    }

    fn collect_targets(
        &mut self,
        edges: &[CurveEdge<I>],
        angle_tolerance_power: u32,
        bounds: &mut CurveBoundsBuffer<I>,
    ) -> bool {
        bounds.build(edges);
        bounds.active.clear();
        self.targets.clear();
        self.targets.resize(edges.len(), false);

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

                self.collect_pair_targets(edges, other.edge_index, current.edge_index, angle_tolerance_power);
            }

            bounds.active.push(current);
        }

        self.targets.iter().any(|&target| target)
    }

    fn collect_pair_targets(
        &mut self,
        edges: &[CurveEdge<I>],
        first_index: usize,
        second_index: usize,
        angle_tolerance_power: u32,
    ) {
        let first = edges[first_index].curve;
        let second = edges[second_index].curve;
        let first_chord = first.chord();
        let second_chord = second.chord();

        if !first.is_nearly_linear(angle_tolerance_power)
            && Self::contains_interior_endpoint(first, second_chord)
        {
            self.targets[first_index] = true;
        }
        if !second.is_nearly_linear(angle_tolerance_power)
            && Self::contains_interior_endpoint(second, first_chord)
        {
            self.targets[second_index] = true;
        }
    }

    fn contains_interior_endpoint(
        curve: crate::kernel::int::curve::segment::Segment<I>,
        other_chord: crate::kernel::int::curve::chord::SegmentChord<I>,
    ) -> bool {
        let chord = curve.chord();
        let hull = curve.convex_hull();

        (other_chord.a != chord.a
            && other_chord.a != chord.b
            && hull.contains_point_border_included(other_chord.a))
            || (other_chord.b != chord.a
                && other_chord.b != chord.b
                && hull.contains_point_border_included(other_chord.b))
    }

    fn build_split_marks(
        &mut self,
        edges: &[CurveEdge<I>],
        subdivision_count: u32,
        min_chord_length_power: u32,
    ) {
        self.split_marks.clear();
        let marks_per_edge = subdivision_count.saturating_sub(1) as usize;
        let target_count = self.targets.iter().filter(|&&target| target).count();
        self.split_marks
            .reserve(target_count.saturating_mul(marks_per_edge));

        for (edge_index, edge) in edges.iter().enumerate() {
            if !self.targets[edge_index] {
                continue;
            }

            let fitting_count =
                Self::fitting_subdivision_count(edge, subdivision_count, min_chord_length_power);
            for index in 1..fitting_count {
                let param = SegmentParam::from_int(I::from_u32(index), I::from_u32(fitting_count));
                self.split_marks.push(CurveSplitMark {
                    edge_index,
                    point: edge.curve.point_at(param),
                    param,
                });
            }
        }
    }

    fn fitting_subdivision_count(
        edge: &CurveEdge<I>,
        requested_count: u32,
        min_chord_length_power: u32,
    ) -> u32 {
        let mut count = requested_count;
        while count > 1 {
            if Self::all_chords_meet_minimum(edge, count, min_chord_length_power) {
                return count;
            }
            count >>= 1;
        }
        1
    }

    fn all_chords_meet_minimum(
        edge: &CurveEdge<I>,
        subdivision_count: u32,
        min_chord_length_power: u32,
    ) -> bool {
        let chord = edge.curve.chord();
        let mut previous = chord.a;
        let min_sqr_length_power = min_chord_length_power.saturating_mul(2);

        for index in 1..=subdivision_count {
            let point = if index == subdivision_count {
                chord.b
            } else {
                let param = SegmentParam::from_int(I::from_u32(index), I::from_u32(subdivision_count));
                edge.curve.point_at(param)
            };
            let sqr_length = (point - previous).sqr_length();
            if sqr_length == I::Wide::ZERO || sqr_length.ilog2() < min_sqr_length_power {
                return false;
            }
            previous = point;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::source::CurveId;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::quad::QuadSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;

    fn line(id: usize, a: [i32; 2], b: [i32; 2]) -> CurveEdge<i32> {
        CurveEdge::full(
            Segment::Line(LineSegment {
                control_points: [a.into(), b.into()],
            }),
            CurveId(id),
        )
    }

    fn quad(id: usize, points: [[i32; 2]; 3]) -> CurveEdge<i32> {
        CurveEdge::full(
            Segment::Quad(QuadSegment {
                control_points: points.map(Into::into),
            }),
            CurveId(id),
        )
    }

    fn refine(edges: &mut Vec<CurveEdge<i32>>, power: u32, iterations: u32, min_length_power: u32) {
        let mut bounds = CurveBoundsBuffer::new();
        CurveContainmentRefiner::new().refine(edges, power, iterations, min_length_power, 5, &mut bounds);
    }

    #[test]
    fn subdivides_containing_curve_into_power_of_two_parts() {
        let mut edges = vec![
            quad(0, [[0, 0], [500, 1000], [1000, 0]]),
            line(1, [500, 800], [900, 800]),
        ];

        refine(&mut edges, 3, 1, 0);

        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 8);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 1);
    }

    #[test]
    fn stops_after_configured_iteration_limit() {
        let mut edges = vec![quad(0, [[0, 0], [8, 16], [16, 0]]), line(1, [8, 12], [9, 12])];

        refine(&mut edges, 2, 2, 0);

        let refined_count = edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count();
        assert!(refined_count >= 4);
        assert!(refined_count <= 16);
    }

    #[test]
    fn does_not_refine_equal_chords() {
        let curve = quad(0, [[0, 0], [5, 10], [10, 0]]);
        let mut edges = vec![curve, CurveEdge::full(curve.curve, CurveId(1))];

        refine(&mut edges, 3, 2, 0);

        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn skips_flat_curve_at_refinement_angle_threshold() {
        let curve = quad(0, [[0, 0], [500, 10], [1000, 0]]);
        let other = line(1, [500, 5], [600, 5]);

        let mut flat_edges = vec![curve, other];
        let mut bounds = CurveBoundsBuffer::new();
        CurveContainmentRefiner::new().refine(&mut flat_edges, 3, 1, 0, 5, &mut bounds);
        assert_eq!(flat_edges.len(), 2);

        let mut curved_edges = vec![curve, other];
        CurveContainmentRefiner::new().refine(&mut curved_edges, 3, 1, 0, 6, &mut bounds);
        assert_eq!(
            curved_edges
                .iter()
                .filter(|edge| edge.curve_id == CurveId(0))
                .count(),
            8
        );
    }

    #[test]
    fn zero_iterations_disable_refinement() {
        let mut edges = vec![quad(0, [[0, 0], [5, 10], [10, 0]]), line(1, [5, 8], [9, 8])];

        refine(&mut edges, 3, 0, 0);

        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn reduces_subdivision_count_to_preserve_minimum_chord_length() {
        let mut edges = vec![quad(0, [[0, 0], [32, 32], [64, 0]]), line(1, [31, 24], [33, 24])];

        refine(&mut edges, 3, 1, 4);

        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 4);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 1);
        assert!(
            edges
                .iter()
                .filter(|edge| edge.curve_id == CurveId(0))
                .all(|edge| edge.curve.chord().sqr_length() >= 256)
        );
    }

    #[test]
    fn skips_refinement_when_two_parts_would_be_too_short() {
        let mut edges = vec![quad(0, [[0, 0], [8, 8], [16, 0]]), line(1, [7, 6], [9, 6])];

        refine(&mut edges, 3, 1, 4);

        assert_eq!(edges.len(), 2);
    }
}
