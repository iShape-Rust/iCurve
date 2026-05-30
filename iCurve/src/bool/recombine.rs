use crate::bool::meta::{MetaSegment, MetaStore, ResolvedCurveOverlay};
use crate::bool::overlay::CurveOverlay;
use crate::curve::arc::EllipticArc;
use crate::curve::contour::CurveContour;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, SegmentParam, SegmentRange,
};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::vector::edge::DataVectorPath;

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(super) fn recombine(&self, resolved: ResolvedCurveOverlay<I, P::Scalar>) -> Vec<CurveShape<P>> {
        let ResolvedCurveOverlay {
            shapes: vector_shapes,
            store,
        } = resolved;
        let mut shapes = Vec::with_capacity(vector_shapes.len());

        for vector_shape in vector_shapes {
            let mut contours = Vec::with_capacity(vector_shape.len());

            for vector_path in vector_shape {
                if let Some(contour) = self.recombine_path(vector_path, &store) {
                    contours.push(contour);
                }
            }

            if !contours.is_empty() {
                shapes.push(CurveShape { contours });
            }
        }

        shapes
    }

    fn recombine_path(
        &self,
        vector_path: DataVectorPath<I, MetaSegment<P::Scalar>>,
        store: &MetaStore<P::Scalar>,
    ) -> Option<CurveContour<P>> {
        let ranges = merge_segment_sets(
            vector_path
                .into_iter()
                .map(|edge| store.to_vec(edge.data))
                .collect(),
        );
        let mut start = None;
        let mut segments = Vec::with_capacity(ranges.len());

        for range in ranges {
            let normalized = &self.segments[range.segment_index].normalized_segment;
            let Some((piece_start, piece)) = normalized.to_curve_piece(range) else {
                continue;
            };

            if start.is_none() {
                start = Some(piece_start);
            }
            segments.push(piece);
        }

        if segments.is_empty() {
            return None;
        }

        Some(CurveContour {
            start: start.expect("non-empty segment list must set contour start"),
            segments,
        })
    }
}

fn merge_segment_sets<F: FloatNumber>(sets: Vec<Vec<SegmentRange<F>>>) -> Vec<SegmentRange<F>> {
    let mut ranges = narrow_segment_sets(sets)
        .into_iter()
        .filter_map(|set| set.first().copied())
        .collect();

    merge_segment_ranges(&mut ranges);
    ranges
}

fn narrow_segment_sets<F: FloatNumber>(mut sets: Vec<Vec<SegmentRange<F>>>) -> Vec<Vec<SegmentRange<F>>> {
    if sets.len() < 2 {
        return sets;
    }

    for i in 1..sets.len() {
        if let Some((narrow_lhs, narrow_rhs)) = intersect_adjacent_sets(&sets[i - 1], &sets[i]) {
            sets[i - 1] = narrow_lhs;
            sets[i] = narrow_rhs;
        }
    }

    if sets.len() > 1 {
        let last_index = sets.len() - 1;
        if let Some((narrow_last, narrow_first)) = intersect_adjacent_sets(&sets[last_index], &sets[0]) {
            sets[last_index] = narrow_last;
            sets[0] = narrow_first;
        }
    }

    sets
}

fn intersect_adjacent_sets<F: FloatNumber>(
    lhs: &[SegmentRange<F>],
    rhs: &[SegmentRange<F>],
) -> Option<(Vec<SegmentRange<F>>, Vec<SegmentRange<F>>)> {
    let mut lhs_out = Vec::new();
    let mut rhs_out = Vec::new();

    for l in lhs {
        for r in rhs {
            if can_merge_ranges(l, r) {
                push_unique(&mut lhs_out, *l);
                push_unique(&mut rhs_out, *r);
            }
        }
    }

    if lhs_out.is_empty() {
        None
    } else {
        Some((lhs_out, rhs_out))
    }
}

fn push_unique<F: FloatNumber>(set: &mut Vec<SegmentRange<F>>, segment: SegmentRange<F>) {
    if !set.contains(&segment) {
        set.push(segment);
    }
}

fn merge_segment_ranges<F: FloatNumber>(ranges: &mut Vec<SegmentRange<F>>) {
    if ranges.len() < 2 {
        return;
    }

    let mut merged: Vec<SegmentRange<F>> = Vec::with_capacity(ranges.len());
    for range in ranges.drain(..) {
        if let Some(last) = merged.last_mut() {
            if can_merge_ranges(last, &range) {
                last.t1 = range.t1;
                continue;
            }
        }

        merged.push(range);
    }

    if merged.len() > 1 {
        let first = merged[0];
        let last_index = merged.len() - 1;
        if can_merge_ranges(&merged[last_index], &first) {
            merged[last_index].t1 = first.t1;
            merged.remove(0);
        }
    }

    *ranges = merged;
}

fn can_merge_ranges<F: FloatNumber>(prev: &SegmentRange<F>, next: &SegmentRange<F>) -> bool {
    prev.segment_index == next.segment_index && prev.t1 == next.t0
}

trait CurvePiece<P: FloatPointCompatible> {
    fn to_curve_piece(&self, range: SegmentRange<P::Scalar>) -> Option<(P, CurveSegment<P>)>;
}

impl<P: FloatPointCompatible> CurvePiece<P> for NormalizedSegment<P> {
    fn to_curve_piece(&self, range: SegmentRange<P::Scalar>) -> Option<(P, CurveSegment<P>)> {
        match self {
            Self::Line(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                Some((
                    segment.control_points[0],
                    CurveSegment::Line {
                        to: segment.control_points[1],
                    },
                ))
            }
            Self::Quad(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                Some((
                    segment.control_points[0],
                    CurveSegment::Quad {
                        ctrl: segment.control_points[1],
                        to: segment.control_points[2],
                    },
                ))
            }
            Self::Cubic(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                Some((
                    segment.control_points[0],
                    CurveSegment::Cubic {
                        ctrl0: segment.control_points[1],
                        ctrl1: segment.control_points[2],
                        to: segment.control_points[3],
                    },
                ))
            }
            Self::Arc(segment) => {
                if range.t0 == range.t1 {
                    return None;
                }

                let t0 = range.t0.value();
                let t1 = range.t1.value();
                let start = segment.point_at(range.t0);
                let arc = EllipticArc {
                    center: segment.center,
                    radii: segment.radii,
                    rotation: segment.rotation,
                    start_angle: segment.start_angle + segment.sweep_angle * t0,
                    sweep_angle: segment.sweep_angle * (t1 - t0),
                };

                Some((start, CurveSegment::Arc { arc }))
            }
        }
    }
}

trait SegmentRangeExtract<P: FloatPointCompatible>:
    SplitAt<P::Scalar, Output = [Self; 2]> + Copy + Sized
{
    fn range(&self, t0: SegmentParam<P::Scalar>, t1: SegmentParam<P::Scalar>) -> Option<Self> {
        if t0 == t1 {
            return None;
        }

        if t0.value() < t1.value() {
            Some(self.forward_range(t0, t1))
        } else {
            Some(self.forward_range(t1, t0).reversed())
        }
    }

    fn forward_range(&self, t0: SegmentParam<P::Scalar>, t1: SegmentParam<P::Scalar>) -> Self {
        if t0 == SegmentParam::Start && t1 == SegmentParam::End {
            return *self;
        }

        if t0 == SegmentParam::Start {
            let [segment, _] = self.split_at(t1.value());
            return segment;
        }

        let [_, right] = self.split_at(t0.value());
        if t1 == SegmentParam::End {
            return right;
        }

        let one = P::Scalar::from_float(1.0);
        let t0 = t0.value();
        let local_t = (t1.value() - t0) / (one - t0);
        let [segment, _] = right.split_at(local_t);
        segment
    }

    fn reversed(self) -> Self;
}

impl<P: FloatPointCompatible> SegmentRangeExtract<P> for LineSegment<P> {
    fn reversed(self) -> Self {
        let [p0, p1] = self.control_points;
        Self {
            control_points: [p1, p0],
        }
    }
}

impl<P: FloatPointCompatible> SegmentRangeExtract<P> for QuadSegment<P> {
    fn reversed(self) -> Self {
        let [p0, p1, p2] = self.control_points;
        Self {
            control_points: [p2, p1, p0],
        }
    }
}

impl<P: FloatPointCompatible> SegmentRangeExtract<P> for CubicSegment<P> {
    fn reversed(self) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        Self {
            control_points: [p3, p2, p1, p0],
        }
    }
}

trait ArcPointAt<P: FloatPointCompatible> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P;
}

impl<P: FloatPointCompatible> ArcPointAt<P> for ArcSegment<P> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P {
        if t == SegmentParam::Start {
            return self.p0;
        }
        if t == SegmentParam::End {
            return self.p1;
        }

        let t = t.value();
        let angle = self.start_angle + self.sweep_angle * t;
        let x = self.radii.x() * angle.cos();
        let y = self.radii.y() * angle.sin();
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        P::from_xy(
            self.center.x() + x * cos - y * sin,
            self.center.y() + x * sin + y * cos,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(segment_index: usize, t0: f64, t1: f64) -> SegmentRange<f64> {
        SegmentRange::new(segment_index, t0, t1)
    }

    #[test]
    fn merge_adjacent_segment_ranges() {
        let ranges = Vec::from([range(1, 0.0, 0.25), range(1, 0.25, 0.75), range(1, 0.75, 1.0)]);

        let mut merged = ranges;
        merge_segment_ranges(&mut merged);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].segment_index, 1);
        assert_eq!(merged[0].t0, SegmentParam::Start);
        assert_eq!(merged[0].t1, SegmentParam::End);
    }

    #[test]
    fn merge_wrapped_segment_ranges() {
        let ranges = Vec::from([range(1, 0.5, 1.0), range(2, 0.0, 1.0), range(1, 0.0, 0.5)]);

        let mut merged = ranges;
        merge_segment_ranges(&mut merged);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].segment_index, 2);
        assert_eq!(merged[1].segment_index, 1);
        assert_eq!(merged[1].t0, SegmentParam::Start);
        assert_eq!(merged[1].t1, SegmentParam::End);
    }

    #[test]
    fn do_not_merge_different_segments() {
        let ranges = Vec::from([range(1, 0.0, 0.5), range(2, 0.5, 1.0)]);

        let mut merged = ranges;
        merge_segment_ranges(&mut merged);

        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn narrow_segment_sets_by_adjacent_intersection() {
        let sets = Vec::from([
            Vec::from([range(1, 0.0, 0.5), range(2, 0.0, 0.5), range(3, 0.0, 0.5)]),
            Vec::from([range(2, 0.5, 0.75), range(3, 0.5, 0.75), range(5, 0.5, 0.75)]),
            Vec::from([range(2, 0.75, 1.0), range(4, 0.75, 1.0), range(5, 0.75, 1.0)]),
        ]);

        let narrowed = narrow_segment_sets(sets);

        assert_eq!(narrowed[0].len(), 2);
        assert!(
            narrowed[0]
                .iter()
                .all(|s| s.segment_index == 2 || s.segment_index == 3)
        );
        assert_eq!(narrowed[1].len(), 1);
        assert_eq!(narrowed[1][0].segment_index, 2);
        assert_eq!(narrowed[2].len(), 1);
        assert_eq!(narrowed[2][0].segment_index, 2);
    }
}
