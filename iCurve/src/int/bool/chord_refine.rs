use crate::int::bool::edge::CurveEdge;
use crate::int::bool::slice::CurveSlice;
use crate::int::bool::split::{CurveEdgeSplitter, CurveSplitMark};
use crate::kernel::int::cross::ChordCross;
use crate::kernel::int::curve::arc::ArcSegment;
use crate::kernel::int::curve::chord::{Chord, SegmentChord};
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::segment::Segment;
use alloc::vec::Vec;
use core::cmp::Ordering;
use i_overlay::core::v_segment::VSegment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_float::int::rect::IntRect;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy)]
struct CurveHullBounds<I: IntNumber> {
    edge_index: usize,
    rect: IntRect<I>,
}

pub(crate) struct ChordTopologyRefiner<I: IntNumber> {
    bounds: Vec<CurveHullBounds<I>>,
    active: Vec<CurveHullBounds<I>>,
    split_marks: Vec<CurveSplitMark<I>>,
    split_marks_buffer: Vec<CurveSplitMark<I>>,
    splitter: CurveEdgeSplitter<I>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefineOutcome {
    PlanarityPreserved,
    Replanarize {
        escaped_marks: usize,
        crossed_chords: usize,
    },
}

impl<I: IntNumber> ChordTopologyRefiner<I> {
    pub(crate) fn new() -> Self {
        Self {
            bounds: Vec::new(),
            active: Vec::new(),
            split_marks: Vec::new(),
            split_marks_buffer: Vec::new(),
            splitter: CurveEdgeSplitter::new(),
        }
    }

    pub(crate) fn refine(
        &mut self,
        edges: &mut Vec<CurveEdge<I>>,
        slices: &mut [CurveSlice<I>],
        cross_radius: I::Wide,
    ) -> RefineOutcome {
        if edges.len() < 2 {
            return RefineOutcome::PlanarityPreserved;
        }

        self.build_bounds(edges);
        let crossed_chords = self.collect_split_marks(edges, cross_radius);
        let outcome = Self::outcome(edges, &self.split_marks, crossed_chords);
        self.splitter.split(edges, &self.split_marks, slices);

        outcome
    }

    fn outcome(edges: &[CurveEdge<I>], marks: &[CurveSplitMark<I>], crossed_chords: usize) -> RefineOutcome {
        let escaped_marks = marks
            .iter()
            .filter(|mark| {
                let curve = edges[mark.edge_index].curve;
                let evaluated = Self::point_at(curve, mark.param);
                let dx = (mark.point.x.to_wide() - evaluated.x.to_wide()).unsigned_abs();
                let dy = (mark.point.y.to_wide() - evaluated.y.to_wide()).unsigned_abs();
                // Fixed-point evaluation and endpoint snapping round separately.
                // A one-unit mismatch is therefore not a topology escape.
                (dx > I::WideUInt::ONE || dy > I::WideUInt::ONE)
                    && !curve.convex_hull().contains_point_border_included(mark.point)
            })
            .count();

        if escaped_marks == 0 && crossed_chords == 0 {
            RefineOutcome::PlanarityPreserved
        } else {
            RefineOutcome::Replanarize {
                escaped_marks,
                crossed_chords,
            }
        }
    }

    fn build_bounds(&mut self, edges: &[CurveEdge<I>]) {
        self.bounds.clear();
        self.bounds.reserve(edges.len());

        for (edge_index, edge) in edges.iter().enumerate() {
            let hull = edge.curve.convex_hull();
            let rect = IntRect::with_points(hull.as_slice()).unwrap();
            self.bounds.push(CurveHullBounds { edge_index, rect });
        }

        self.bounds
            .sort_unstable_by_key(|item| (item.rect.min_x, item.rect.min_y));
    }

    fn collect_split_marks(&mut self, edges: &[CurveEdge<I>], cross_radius: I::Wide) -> usize {
        self.active.clear();
        self.split_marks.clear();
        let mut crossed_chords = 0;

        for bounds_index in 0..self.bounds.len() {
            let current = self.bounds[bounds_index];
            self.active.retain(|other| other.rect.max_x >= current.rect.min_x);

            for active_index in 0..self.active.len() {
                let other = self.active[active_index];
                if !other.rect.is_intersect_border_include(&current.rect) {
                    continue;
                }

                crossed_chords += self.collect_pair_marks(edges, other, current, cross_radius);
            }

            self.active.push(current);
        }

        CurveSplitMark::sort_and_dedup(&mut self.split_marks, &mut self.split_marks_buffer);
        crossed_chords
    }

    fn collect_pair_marks(
        &mut self,
        edges: &[CurveEdge<I>],
        first: CurveHullBounds<I>,
        second: CurveHullBounds<I>,
        cross_radius: I::Wide,
    ) -> usize {
        let first_edge = edges[first.edge_index];
        let second_edge = edges[second.edge_index];
        let first_chord = first_edge.curve.chord();
        let second_chord = second_edge.curve.chord();

        if Self::chords_coincide(first_chord, second_chord) {
            self.collect_coincident_chord_mark(
                first.edge_index,
                first_edge.curve,
                second.edge_index,
                second_edge.curve,
            );
            return 0;
        }

        if let Some(ChordCross::Point(point)) = first_chord.cross(&second_chord, cross_radius) {
            let snap_to_endpoint = point == first_chord.a
                || point == first_chord.b
                || point == second_chord.a
                || point == second_chord.b;
            let first_split = self.collect_chord_cross_mark(
                first.edge_index,
                first_edge.curve,
                first_chord,
                point,
                snap_to_endpoint,
            );
            let second_split = self.collect_chord_cross_mark(
                second.edge_index,
                second_edge.curve,
                second_chord,
                point,
                snap_to_endpoint,
            );

            if first_split || second_split {
                return 1;
            }
        }

        self.collect_endpoint_mark(second.edge_index, second_edge.curve, second.rect, first_chord.a);
        self.collect_endpoint_mark(second.edge_index, second_edge.curve, second.rect, first_chord.b);
        self.collect_endpoint_mark(first.edge_index, first_edge.curve, first.rect, second_chord.a);
        self.collect_endpoint_mark(first.edge_index, first_edge.curve, first.rect, second_chord.b);

        0
    }

    fn collect_chord_cross_mark(
        &mut self,
        edge_index: usize,
        curve: Segment<I>,
        chord: SegmentChord<I>,
        cross_point: IntPoint<I>,
        snap_to_endpoint: bool,
    ) -> bool {
        let denominator = SegmentParam::<I>::DENOMINATOR;
        let mut param = chord.param_for_point(cross_point);
        if param.value() <= I::Wide::ZERO || param.value() >= denominator {
            return false;
        }

        let mut point = if snap_to_endpoint {
            cross_point
        } else {
            Self::point_at(curve, param)
        };
        if !snap_to_endpoint && (point == chord.a || point == chord.b) {
            param = SegmentParam::half();
            point = Self::point_at(curve, param);
            if point == chord.a || point == chord.b {
                return false;
            }
        }

        self.split_marks.push(CurveSplitMark {
            edge_index,
            point,
            param,
        });
        true
    }

    fn collect_endpoint_mark(
        &mut self,
        edge_index: usize,
        curve: Segment<I>,
        bounds: IntRect<I>,
        point: IntPoint<I>,
    ) {
        let chord = curve.chord();
        let min_x = chord.a.x.min(chord.b.x);
        let max_x = chord.a.x.max(chord.b.x);
        if point.x <= min_x || point.x >= max_x || !bounds.contains(point) {
            return;
        }

        let hull = curve.convex_hull();
        if !hull.contains_point_border_included(point) {
            return;
        }

        if let Some((param, split_point)) = Self::sample_at_x(curve, point.x) {
            CurveSplitMark::push_if_interior(&mut self.split_marks, edge_index, split_point, param);
        }
    }

    fn collect_coincident_chord_mark(
        &mut self,
        first_index: usize,
        first: Segment<I>,
        second_index: usize,
        second: Segment<I>,
    ) {
        if Self::has_same_geometry(first, second) {
            return;
        }

        let chord = first.chord();
        let (start, end) = if chord.a < chord.b {
            (chord.a, chord.b)
        } else {
            (chord.b, chord.a)
        };
        let reference = VSegment::new(start, end);
        let first_tangent = VSegment::new(start, Self::initial_tangent_end(first, start));
        let second_tangent = VSegment::new(start, Self::initial_tangent_end(second, start));
        let curve_order = first_tangent.cmp_by_angle(&second_tangent);

        if curve_order != Ordering::Equal {
            if first_tangent.cmp_by_angle(&reference) == curve_order
                && let Some((param, point)) =
                    Self::find_ordered_split(first, start, reference, curve_order, true)
            {
                CurveSplitMark::push_if_interior(&mut self.split_marks, first_index, point, param);
                return;
            }

            if reference.cmp_by_angle(&second_tangent) == curve_order
                && let Some((param, point)) =
                    Self::find_ordered_split(second, start, reference, curve_order, false)
            {
                CurveSplitMark::push_if_interior(&mut self.split_marks, second_index, point, param);
                return;
            }
        }

        Self::push_half_mark(&mut self.split_marks, first_index, first);
        Self::push_half_mark(&mut self.split_marks, second_index, second);
    }

    fn find_ordered_split(
        curve: Segment<I>,
        start: IntPoint<I>,
        reference: VSegment<I>,
        required_order: Ordering,
        candidate_is_first: bool,
    ) -> Option<(SegmentParam<I>, IntPoint<I>)> {
        let chord = curve.chord();
        let forward = chord.a == start;
        let denominator = SegmentParam::<I>::DENOMINATOR;
        let mut offset = denominator >> 1;

        while offset > I::Wide::ZERO {
            let value = if forward { offset } else { denominator - offset };
            let param = SegmentParam::new(I::from_wide(value));
            let point = Self::point_at(curve, param);

            if point != start && point != reference.b && point > start {
                let candidate = VSegment::new(start, point);
                let order = if candidate_is_first {
                    candidate.cmp_by_angle(&reference)
                } else {
                    reference.cmp_by_angle(&candidate)
                };
                if order == required_order {
                    return Some((param, point));
                }
            }

            offset = offset >> 1;
        }

        None
    }

    fn push_half_mark(marks: &mut Vec<CurveSplitMark<I>>, edge_index: usize, curve: Segment<I>) {
        let param = SegmentParam::half();
        let point = Self::point_at(curve, param);
        CurveSplitMark::push_if_interior(marks, edge_index, point, param);
    }

    fn sample_at_x(curve: Segment<I>, x: I) -> Option<(SegmentParam<I>, IntPoint<I>)> {
        let chord = curve.chord();
        let increasing = chord.a.x < chord.b.x;
        if !increasing && chord.a.x == chord.b.x {
            return None;
        }

        let denominator = SegmentParam::<I>::DENOMINATOR;
        let mut low = I::Wide::ZERO;
        let mut high = denominator;

        while high - low > I::Wide::ONE {
            let value = low + ((high - low) >> 1);
            let param = SegmentParam::new(I::from_wide(value));
            let point = Self::point_at(curve, param);

            if point.x == x {
                let mut split_point = point;
                split_point.x = x;
                return Some((param, split_point));
            }

            let before = if increasing { point.x < x } else { point.x > x };
            if before {
                low = value;
            } else {
                high = value;
            }
        }

        let low = low.max(I::Wide::ONE);
        let high = high.min(denominator - I::Wide::ONE);
        if low > high {
            return None;
        }

        let low_param = SegmentParam::new(I::from_wide(low));
        let high_param = SegmentParam::new(I::from_wide(high));
        let low_point = Self::point_at(curve, low_param);
        let high_point = Self::point_at(curve, high_param);
        let low_distance = (low_point.x.to_wide() - x.to_wide()).unsigned_abs();
        let high_distance = (high_point.x.to_wide() - x.to_wide()).unsigned_abs();
        let (param, mut point) = if low_distance <= high_distance {
            (low_param, low_point)
        } else {
            (high_param, high_point)
        };
        point.x = x;

        Some((param, point))
    }

    #[inline]
    fn point_at(curve: Segment<I>, param: SegmentParam<I>) -> IntPoint<I> {
        curve.point_at(param)
    }

    fn initial_tangent_end(curve: Segment<I>, start: IntPoint<I>) -> IntPoint<I> {
        match curve {
            Segment::Line(line) => Self::initial_tangent_end_in(&line.control_points, start),
            Segment::Quad(quad) => Self::initial_tangent_end_in(&quad.control_points, start),
            Segment::Cubic(cubic) => Self::initial_tangent_end_in(&cubic.control_points, start),
            Segment::Arc(arc) => Self::initial_tangent_end_in(&arc.control_points, start),
        }
    }

    fn initial_tangent_end_in(points: &[IntPoint<I>], start: IntPoint<I>) -> IntPoint<I> {
        if points[0] == start {
            points.iter().copied().find(|&point| point != start).unwrap()
        } else {
            points
                .iter()
                .rev()
                .copied()
                .find(|&point| point != start)
                .unwrap()
        }
    }

    #[inline]
    fn chords_coincide(first: SegmentChord<I>, second: SegmentChord<I>) -> bool {
        first.a == second.a && first.b == second.b || first.a == second.b && first.b == second.a
    }

    fn has_same_geometry(first: Segment<I>, second: Segment<I>) -> bool {
        let first_chord = first.chord();
        let second_chord = second.chord();
        let reverse_second = first_chord.a == second_chord.b && first_chord.b == second_chord.a;

        if let (Segment::Arc(first), Segment::Arc(second)) = (first, second) {
            return Self::has_same_arc_geometry(first, second, reverse_second);
        }

        let Some(first_controls) = Self::scaled_cubic_controls(first, false) else {
            return false;
        };
        let Some(second_controls) = Self::scaled_cubic_controls(second, reverse_second) else {
            return false;
        };

        first_controls == second_controls
    }

    fn has_same_arc_geometry(first: ArcSegment<I>, second: ArcSegment<I>, reverse_second: bool) -> bool {
        let mut second_controls = second.control_points;
        let mut second_weights = second.weights;
        if reverse_second {
            second_controls.reverse();
            second_weights.reverse();
        }

        if first.control_points != second_controls {
            return false;
        }

        // Rational control nets are homogeneous: multiplying all weights by
        // one common positive factor does not change the represented curve.
        let first_anchor = first.weights[0].to_wide();
        let second_anchor = second_weights[0].to_wide();
        first
            .weights
            .into_iter()
            .zip(second_weights)
            .all(|(a, b)| a.to_wide() * second_anchor == b.to_wide() * first_anchor)
    }

    fn scaled_cubic_controls(curve: Segment<I>, reverse: bool) -> Option<[(I::Wide, I::Wide); 4]> {
        let two = I::Wide::TWO;
        let three = I::Wide::from_u32(3);
        let scale =
            |point: IntPoint<I>, factor: I::Wide| (factor * point.x.to_wide(), factor * point.y.to_wide());
        let combine = |a: IntPoint<I>, a_factor: I::Wide, b: IntPoint<I>, b_factor: I::Wide| {
            (
                a_factor * a.x.to_wide() + b_factor * b.x.to_wide(),
                a_factor * a.y.to_wide() + b_factor * b.y.to_wide(),
            )
        };

        let mut controls = match curve {
            Segment::Line(line) => {
                let [p0, p1] = line.control_points;
                [
                    scale(p0, three),
                    combine(p0, two, p1, I::Wide::ONE),
                    combine(p0, I::Wide::ONE, p1, two),
                    scale(p1, three),
                ]
            }
            Segment::Quad(quad) => {
                let [p0, p1, p2] = quad.control_points;
                [
                    scale(p0, three),
                    combine(p0, I::Wide::ONE, p1, two),
                    combine(p1, two, p2, I::Wide::ONE),
                    scale(p2, three),
                ]
            }
            Segment::Cubic(cubic) => cubic.control_points.map(|point| scale(point, three)),
            Segment::Arc(arc) => {
                let [w0, w1, w2] = arc.weights;
                if w0 != w1 || w1 != w2 {
                    return None;
                }

                let [p0, p1, p2] = arc.control_points;
                [
                    scale(p0, three),
                    combine(p0, I::Wide::ONE, p1, two),
                    combine(p1, two, p2, I::Wide::ONE),
                    scale(p2, three),
                ]
            }
        };

        if reverse {
            controls.reverse();
        }
        Some(controls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::bool::slice::{CurveId, CurveSlice};
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
    use crate::kernel::int::curve::cubic::CubicSegment;
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::quad::QuadSegment;
    use alloc::vec;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;

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

    fn quarter_arc() -> ArcSegment<i32> {
        let one = FixedScale::<i32>::DENOMINATOR as i32;

        ArcSegment {
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
        }
    }

    fn slices(edges: &[CurveEdge<i32>]) -> Vec<CurveSlice<i32>> {
        edges
            .iter()
            .map(|edge| CurveSlice::new(edge.curve, ShapeType::Subject))
            .collect()
    }

    #[test]
    fn splits_curve_when_other_chord_endpoint_is_inside_its_hull() {
        let mut edges = vec![quad(0, [[0, 0], [5, 10], [10, 0]]), line(1, [5, 8], [9, 8])];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        refiner.refine(&mut edges, &mut slices, 0_i64);

        assert_eq!(edges.len(), 3);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 1);
        assert_eq!(slices[0].marks.len(), 3);
        for edge in edges.iter().filter(|edge| edge.curve_id == CurveId(0)) {
            let chord = edge.curve.chord();
            assert!(slices[0].param_at(chord.a).is_some());
            assert!(slices[0].param_at(chord.b).is_some());
        }
    }

    #[test]
    fn splits_only_upper_curve_for_distinct_coincident_chords_above_chord() {
        let mut edges = vec![
            quad(0, [[0, 0], [5, 10], [10, 0]]),
            quad(1, [[0, 0], [5, 5], [10, 0]]),
        ];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        refiner.refine(&mut edges, &mut slices, 0_i64);

        assert_eq!(edges.len(), 3);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 1);
    }

    #[test]
    fn splits_only_lower_curve_for_distinct_coincident_chords_below_chord() {
        let mut edges = vec![
            quad(0, [[0, 0], [5, -5], [10, 0]]),
            quad(1, [[0, 0], [5, -10], [10, 0]]),
        ];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        refiner.refine(&mut edges, &mut slices, 0_i64);

        assert_eq!(edges.len(), 3);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 1);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
    }

    #[test]
    fn keeps_identical_curves_with_coincident_chords() {
        let curve = quad(0, [[0, 0], [5, 10], [10, 0]]);
        let mut edges = vec![curve, CurveEdge::full(curve.curve, CurveId(1))];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        refiner.refine(&mut edges, &mut slices, 0_i64);

        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn keeps_identical_reversed_curves_with_coincident_chords() {
        let mut edges = vec![
            quad(0, [[0, 0], [5, 10], [10, 0]]),
            quad(1, [[10, 0], [5, 10], [0, 0]]),
        ];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        refiner.refine(&mut edges, &mut slices, 0_i64);

        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn recognizes_equal_geometry_across_segment_degrees() {
        let line = Segment::Line(LineSegment {
            control_points: [[0, 0].into(), [3, 0].into()],
        });
        let cubic = Segment::Cubic(CubicSegment {
            control_points: [[0, 0].into(), [1, 0].into(), [2, 0].into(), [3, 0].into()],
        });

        assert!(ChordTopologyRefiner::has_same_geometry(line, cubic));
    }

    #[test]
    fn arc_initial_tangent_uses_first_distinct_control() {
        let mut arc = quarter_arc();

        assert_eq!(
            ChordTopologyRefiner::initial_tangent_end(Segment::Arc(arc), arc.control_points[0]),
            arc.control_points[1]
        );
        assert_eq!(
            ChordTopologyRefiner::initial_tangent_end(Segment::Arc(arc), arc.control_points[2]),
            arc.control_points[1]
        );

        arc.control_points[1] = arc.control_points[0];
        assert_eq!(
            ChordTopologyRefiner::initial_tangent_end(Segment::Arc(arc), arc.control_points[0]),
            arc.control_points[2]
        );
    }

    #[test]
    fn recognizes_identical_and_reversed_arc_geometry() {
        let arc = quarter_arc();

        assert!(ChordTopologyRefiner::has_same_geometry(
            Segment::Arc(arc),
            Segment::Arc(arc)
        ));
        assert!(ChordTopologyRefiner::has_same_geometry(
            Segment::Arc(arc),
            Segment::Arc(arc.reversed())
        ));
    }

    #[test]
    fn rational_arc_geometry_accepts_common_weight_scale_only() {
        let mut first = quarter_arc();
        first.weights = [100, 80, 100];
        let mut proportional = first;
        proportional.weights = [50, 40, 50];
        let mut different = first;
        different.weights = [100, 79, 100];

        assert!(ChordTopologyRefiner::has_same_geometry(
            Segment::Arc(first),
            Segment::Arc(proportional)
        ));
        assert!(!ChordTopologyRefiner::has_same_geometry(
            Segment::Arc(first),
            Segment::Arc(different)
        ));
    }

    #[test]
    fn equal_weight_arc_can_match_polynomial_quad() {
        let mut arc = quarter_arc();
        arc.weights = [100, 100, 100];
        let quad = Segment::Quad(QuadSegment {
            control_points: arc.control_points,
        });

        assert!(ChordTopologyRefiner::has_same_geometry(Segment::Arc(arc), quad));
    }

    #[test]
    fn bisects_both_distinct_curves_when_initial_tangents_coincide() {
        let mut edges = vec![
            CurveEdge::full(
                Segment::Cubic(CubicSegment {
                    control_points: [[0, 0].into(), [2, 4].into(), [8, 4].into(), [10, 0].into()],
                }),
                CurveId(0),
            ),
            CurveEdge::full(
                Segment::Cubic(CubicSegment {
                    control_points: [[0, 0].into(), [4, 8].into(), [8, 2].into(), [10, 0].into()],
                }),
                CurveId(1),
            ),
        ];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        refiner.refine(&mut edges, &mut slices, 0_i64);

        assert_eq!(edges.len(), 4);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
    }

    #[test]
    fn splits_both_curves_when_their_chords_cross() {
        let mut edges = vec![
            quad(0, [[0, 120], [-55, 120], [-110, 84]]),
            quad(1, [[-14, 124], [-69, 124], [-110, 55]]),
        ];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        let outcome = refiner.refine(&mut edges, &mut slices, 0_i64);

        assert_eq!(
            outcome,
            RefineOutcome::Replanarize {
                escaped_marks: 0,
                crossed_chords: 1,
            }
        );
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 2);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
    }

    #[test]
    fn snaps_near_chord_crossing_to_endpoint_with_radius() {
        let endpoint = IntPoint::new(0, 0);
        let mut edges = vec![line(0, [0, 0], [10, 3]), line(1, [1, -10], [1, 10])];
        let mut slices = slices(&edges);
        let mut refiner = ChordTopologyRefiner::new();

        let outcome = refiner.refine(&mut edges, &mut slices, 1_i64);

        assert_eq!(
            outcome,
            RefineOutcome::Replanarize {
                escaped_marks: 0,
                crossed_chords: 1,
            }
        );
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(0)).count(), 1);
        assert_eq!(edges.iter().filter(|edge| edge.curve_id == CurveId(1)).count(), 2);
        assert!(
            edges
                .iter()
                .filter(|edge| edge.curve_id == CurveId(1))
                .all(|edge| {
                    let chord = edge.curve.chord();
                    chord.a == endpoint || chord.b == endpoint
                })
        );
    }

    #[test]
    fn requests_replanarization_when_mark_escapes_source_hull() {
        let edges = vec![quad(0, [[0, 0], [5, 10], [10, 0]])];
        let marks = vec![CurveSplitMark {
            edge_index: 0,
            point: IntPoint::new(5, 11),
            param: SegmentParam::half(),
        }];

        assert_eq!(
            ChordTopologyRefiner::outcome(&edges, &marks, 0),
            RefineOutcome::Replanarize {
                escaped_marks: 1,
                crossed_chords: 0,
            }
        );
    }

    #[test]
    fn accepts_one_unit_snap_outside_discrete_hull() {
        let edges = vec![line(0, [0, 0], [2, 2])];
        let marks = vec![CurveSplitMark {
            edge_index: 0,
            point: IntPoint::new(1, 2),
            param: SegmentParam::half(),
        }];

        assert_eq!(
            ChordTopologyRefiner::outcome(&edges, &marks, 0),
            RefineOutcome::PlanarityPreserved
        );
    }

    #[test]
    fn preserves_planarity_when_all_marks_stay_in_source_hulls() {
        let edges = vec![quad(0, [[0, 0], [5, 10], [10, 0]])];
        let marks = vec![CurveSplitMark {
            edge_index: 0,
            point: IntPoint::new(5, 5),
            param: SegmentParam::half(),
        }];

        assert_eq!(
            ChordTopologyRefiner::outcome(&edges, &marks, 0),
            RefineOutcome::PlanarityPreserved
        );
    }
}
