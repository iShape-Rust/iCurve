mod span;

use crate::bool::meta::{MetaSegment, MetaStore, ResolvedCurveOverlay};
use crate::bool::overlay::CurveOverlay;
use crate::bool::recombine::span::CurveSpan;
use crate::collections::circular_merge_list::CircularMergeList;
use crate::curve::arc::EllipticArc;
use crate::curve::contour::CurveContour;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, Segment, SegmentParam,
    SegmentRange,
};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::point::IntPoint;
use i_overlay::vector::edge::DataVectorPath;

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(super) fn recombine(&self, resolved: ResolvedCurveOverlay<I, P::Scalar>) -> Vec<CurveShape<P>> {
        let ResolvedCurveOverlay {
            shapes: vector_shapes,
            store,
        } = resolved;
        let mut shapes = Vec::with_capacity(vector_shapes.len());
        let mut merge_list = CircularMergeList::with_capacity(0);

        for vector_shape in vector_shapes {
            let mut contours = Vec::with_capacity(vector_shape.len());

            for vector_path in vector_shape {
                if let Some(contour) = self.recombine_path(vector_path, &store, &mut merge_list) {
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
        merge_list: &mut CircularMergeList<SegmentData<P::Scalar, I>>,
    ) -> Option<CurveContour<P>> {
        let sets = vector_path
            .into_iter()
            .map(|edge| SegmentData {
                ranges: store.range_iter(edge.data).collect(),
                start: edge.a,
                end: edge.b,
            })
            .collect();

        let ranges = merge_segment_sets(sets, &self.segments, &self.adapter, merge_list);
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

#[derive(Clone)]
struct SegmentData<F: FloatNumber, I: IntNumber> {
    ranges: Vec<SegmentRange<F>>,
    start: IntPoint<I>,
    end: IntPoint<I>,
}

fn merge_segment_sets<P, I>(
    sets: Vec<SegmentData<P::Scalar, I>>,
    segments: &[Segment<P>],
    adapter: &FloatPointAdapter<P, I>,
    merge_list: &mut CircularMergeList<SegmentData<P::Scalar, I>>,
) -> Vec<SegmentRange<P::Scalar>>
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    merge_list
        .merge_with(sets, |prev, next| prev.merge(next, segments, adapter))
        .into_iter()
        .filter_map(|set| set.ranges.first().copied())
        .collect()
}

fn can_merge_ranges<F: FloatNumber>(prev: &SegmentRange<F>, next: &SegmentRange<F>) -> bool {
    prev.segment_index == next.segment_index && prev.t1 == next.t0
}

impl<F: FloatNumber, I: IntNumber> SegmentData<F, I> {
    fn merge<P>(
        &mut self,
        other: &mut Self,
        segments: &[Segment<P>],
        adapter: &FloatPointAdapter<P, I>,
    ) -> bool
    where
        P: FloatPointCompatible<Scalar = F>,
    {
        if self.merge_by_index(other) {
            return true;
        }

        self.merge_by_geometry(other, segments, adapter)
    }

    fn merge_by_index(&mut self, other: &Self) -> bool {
        let mut result = Vec::new();

        for l in self.ranges.iter() {
            for r in other.ranges.iter() {
                if can_merge_ranges(l, r) {
                    let mut range = *l;
                    range.t1 = r.t1;

                    if !result.contains(&range) {
                        result.push(range);
                    }
                }
            }
        }

        if result.is_empty() {
            false
        } else {
            self.ranges = result;
            self.end = other.end;
            true
        }
    }

    fn merge_by_geometry<P>(
        &mut self,
        other: &Self,
        segments: &[Segment<P>],
        adapter: &FloatPointAdapter<P, I>,
    ) -> bool
    where
        P: FloatPointCompatible<Scalar = F>,
    {
        let mut result = Vec::new();

        for l in self.ranges.iter() {
            for r in other.ranges.iter() {
                let prev_segment = &segments[l.segment_index].normalized_segment;
                let next_segment = &segments[r.segment_index].normalized_segment;

                let prev = CurveSpan::new(self.start, self.end, prev_segment, *l);
                let next = CurveSpan::new(other.start, other.end, next_segment, *r);

                if prev.can_recombine_with(next, adapter) {
                    let mut left = *l;
                    left.t1 = r.t1;
                    push_unique(&mut result, left);

                    let mut right = *r;
                    right.t0 = l.t0;
                    push_unique(&mut result, right);
                }
            }
        }

        if result.is_empty() {
            false
        } else {
            self.ranges = result;
            self.end = other.end;
            true
        }
    }
}

fn push_unique<F: FloatNumber>(ranges: &mut Vec<SegmentRange<F>>, range: SegmentRange<F>) {
    if !ranges.contains(&range) {
        ranges.push(range);
    }
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
    use crate::collections::circular_merge_list::Merge;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::i_float::float::rect::FloatRect;

    fn range(segment_index: usize, t0: f64, t1: f64) -> SegmentRange<f64> {
        SegmentRange::new(segment_index, t0, t1)
    }

    fn data(ranges: Vec<SegmentRange<f64>>, x0: i32, x1: i32) -> SegmentData<f64, i32> {
        SegmentData {
            ranges,
            start: IntPoint::new(x0, 0),
            end: IntPoint::new(x1, 0),
        }
    }

    fn line_segments(count: usize) -> Vec<Segment<[f64; 2]>> {
        (0..count)
            .map(|_| Segment {
                normalized_segment: NormalizedSegment::Line(LineSegment {
                    control_points: [[0.0, 0.0], [1.0, 0.0]],
                }),
                shape_type: ShapeType::Subject,
            })
            .collect()
    }

    fn segment(normalized_segment: NormalizedSegment<[f64; 2]>) -> Segment<[f64; 2]> {
        Segment {
            normalized_segment,
            shape_type: ShapeType::Subject,
        }
    }

    impl<F: FloatNumber> Merge for SegmentRange<F> {
        fn merge(&mut self, other: &mut Self) -> bool {
            if can_merge_ranges(self, other) {
                self.t1 = other.t1;
                true
            } else {
                false
            }
        }
    }

    #[test]
    fn merge_adjacent_segment_ranges() {
        let ranges = Vec::from([range(1, 0.0, 0.25), range(1, 0.25, 0.75), range(1, 0.75, 1.0)]);
        let mut merge_list = CircularMergeList::with_capacity(ranges.len());

        let merged = merge_list.merge(ranges);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].segment_index, 1);
        assert_eq!(merged[0].t0, SegmentParam::Start);
        assert_eq!(merged[0].t1, SegmentParam::End);
    }

    #[test]
    fn merge_wrapped_segment_ranges() {
        let ranges = Vec::from([range(1, 0.5, 1.0), range(2, 0.0, 1.0), range(1, 0.0, 0.5)]);
        let mut merge_list = CircularMergeList::with_capacity(ranges.len());

        let merged = merge_list.merge(ranges);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].segment_index, 1);
        assert_eq!(merged[0].t0, SegmentParam::Start);
        assert_eq!(merged[0].t1, SegmentParam::End);
        assert_eq!(merged[1].segment_index, 2);
    }

    #[test]
    fn do_not_merge_different_segments() {
        let ranges = Vec::from([range(1, 0.0, 0.5), range(2, 0.5, 1.0)]);
        let mut merge_list = CircularMergeList::with_capacity(ranges.len());

        let merged = merge_list.merge(ranges);

        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn narrow_segment_sets_by_adjacent_intersection() {
        let sets = Vec::from([
            data(
                Vec::from([range(1, 0.0, 0.5), range(2, 0.0, 0.5), range(3, 0.0, 0.5)]),
                0,
                1,
            ),
            data(
                Vec::from([range(2, 0.5, 0.75), range(3, 0.5, 0.75), range(5, 0.5, 0.75)]),
                1,
                2,
            ),
            data(
                Vec::from([range(2, 0.75, 1.0), range(4, 0.75, 1.0), range(5, 0.75, 1.0)]),
                2,
                3,
            ),
        ]);
        let mut merge_list = CircularMergeList::with_capacity(sets.len());
        let segments = line_segments(6);
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-10.0, 10.0, -10.0, 10.0), 1000.0);

        let ranges = merge_segment_sets(sets, &segments, &adapter, &mut merge_list);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].segment_index, 2);
        assert_eq!(ranges[0].t0, SegmentParam::Start);
        assert_eq!(ranges[0].t1, SegmentParam::End);
    }

    #[test]
    fn geometry_merge_keeps_expanded_line_alternatives() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-10.0, 10.0, -10.0, 10.0), 1000.0);
        let segments = Vec::from([
            segment(NormalizedSegment::Line(LineSegment {
                control_points: [[0.0, 0.0], [2.0, 0.0]],
            })),
            segment(NormalizedSegment::Line(LineSegment {
                control_points: [[0.0, 0.0], [2.0, 0.0]],
            })),
        ]);
        let mut left = data(Vec::from([range(0, 0.0, 0.5)]), 0, 1000);
        let mut right = data(Vec::from([range(1, 0.5, 1.0)]), 1000, 2000);

        assert!(left.merge(&mut right, &segments, &adapter));

        assert_eq!(left.start, IntPoint::new(0, 0));
        assert_eq!(left.end, IntPoint::new(2000, 0));
        assert_eq!(left.ranges, Vec::from([range(0, 0.0, 1.0), range(1, 0.0, 1.0)]));
    }

    #[test]
    fn geometry_merge_keeps_expanded_quad_alternatives() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-10.0, 10.0, -10.0, 10.0), 1000.0);
        let quad = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 4.0], [6.0, 0.0]],
        };
        let segments = Vec::from([
            segment(NormalizedSegment::Quad(quad)),
            segment(NormalizedSegment::Quad(quad)),
        ]);
        let mut left = SegmentData {
            ranges: Vec::from([range(0, 0.0, 0.5)]),
            start: adapter.float_to_int(&[0.0, 0.0]),
            end: adapter.float_to_int(&[2.5, 2.0]),
        };
        let mut right = SegmentData {
            ranges: Vec::from([range(1, 0.5, 1.0)]),
            start: adapter.float_to_int(&[2.5, 2.0]),
            end: adapter.float_to_int(&[6.0, 0.0]),
        };

        assert!(left.merge(&mut right, &segments, &adapter));

        assert_eq!(left.start, adapter.float_to_int(&[0.0, 0.0]));
        assert_eq!(left.end, adapter.float_to_int(&[6.0, 0.0]));
        assert_eq!(left.ranges, Vec::from([range(0, 0.0, 1.0), range(1, 0.0, 1.0)]));
    }
}
