use crate::bool::meta::{MetaSegment, MetaStore, ResolvedCurveOverlay};
use crate::bool::overlay::CurveOverlay;
use crate::collections::circular_merge_list::CircularMergeList;
use crate::curve::path::CurvePath;
use crate::curve::segment::CurveSegment;
use crate::curve::shape::CurveShape;
use crate::flatten::segment::{SegmentRange, ShapeSegment};
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::point_at::PointAt;
use crate::kernel::curve::reversed::Reversed;
use crate::kernel::curve::segment::Segment;
use crate::kernel::curve::split_at::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
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
    ) -> Option<CurvePath<P>> {
        let sets = vector_path
            .into_iter()
            .map(|edge| SegmentData {
                ranges: store.range_iter(edge.data).collect(),
                start: edge.a,
                end: edge.b,
            })
            .collect();

        let point_adapter = self.adapter.to_float_point_adapter();
        let ranges = merge_segment_sets(sets, &self.segments, &point_adapter, merge_list);
        let mut start = None;
        let mut segments = Vec::with_capacity(ranges.len());

        for range in ranges {
            let normalized = &self.segments[range.segment_index].segment;
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

        Some(CurvePath {
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

fn merge_segment_sets<T, I>(
    sets: Vec<SegmentData<T, I>>,
    segments: &[ShapeSegment<T>],
    adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    merge_list: &mut CircularMergeList<SegmentData<T, I>>,
) -> Vec<SegmentRange<T>>
where
    T: FloatNumber,
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

impl<T: FloatNumber, I: IntNumber> SegmentData<T, I> {
    fn merge(
        &mut self,
        other: &mut Self,
        segments: &[ShapeSegment<T>],
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> bool {
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

    fn merge_by_geometry(
        &mut self,
        other: &Self,
        segments: &[ShapeSegment<T>],
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> bool {
        if self.end != other.start {
            return false;
        }

        let mut result = Vec::new();
        for left in &self.ranges {
            for right in &other.ranges {
                if left.segment_index == right.segment_index {
                    continue;
                }

                let left_segment = &segments[left.segment_index].segment;
                let right_segment = &segments[right.segment_index].segment;
                let left_end = adapter.float_to_int(&segment_point_at(left_segment, left.t1));
                let right_start = adapter.float_to_int(&segment_point_at(right_segment, right.t0));
                if left_end != self.end || right_start != other.start {
                    continue;
                }

                let Some(left_full) =
                    full_range_for_geometry(left.segment_index, left_segment, self.start, other.end, adapter)
                else {
                    continue;
                };
                let Some(right_full) = full_range_for_geometry(
                    right.segment_index,
                    right_segment,
                    self.start,
                    other.end,
                    adapter,
                ) else {
                    continue;
                };

                push_unique(&mut result, left_full);
                push_unique(&mut result, right_full);
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

fn full_range_for_geometry<T: FloatNumber, I: IntNumber>(
    segment_index: usize,
    segment: &Segment<T>,
    start: IntPoint<I>,
    end: IntPoint<I>,
    adapter: &FloatPointAdapter<FloatPoint<T>, I>,
) -> Option<SegmentRange<T>> {
    let segment_start = adapter.float_to_int(&segment_point_at(segment, SegmentParam::Start));
    let segment_end = adapter.float_to_int(&segment_point_at(segment, SegmentParam::End));

    if segment_start == start && segment_end == end {
        Some(SegmentRange::full(segment_index))
    } else if segment_start == end && segment_end == start {
        Some(SegmentRange {
            segment_index,
            t0: SegmentParam::End,
            t1: SegmentParam::Start,
        })
    } else {
        None
    }
}

fn segment_point_at<T: FloatNumber>(segment: &Segment<T>, t: SegmentParam<T>) -> FloatPoint<T> {
    match segment {
        Segment::Line(segment) => segment.point_at(t),
        Segment::Quad(segment) => segment.point_at(t),
        Segment::Cubic(segment) => segment.point_at(t),
    }
}

trait CurvePiece<P: FloatPointCompatible> {
    fn to_curve_piece(&self, range: SegmentRange<P::Scalar>) -> Option<(P, CurveSegment<P>)>;
}

impl<P: FloatPointCompatible> CurvePiece<P> for Segment<P::Scalar> {
    fn to_curve_piece(&self, range: SegmentRange<P::Scalar>) -> Option<(P, CurveSegment<P>)> {
        match self {
            Self::Line(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                let p0 = segment.control_points[0];
                let p1 = segment.control_points[1];
                Some((
                    P::from_xy(p0.x, p0.y),
                    CurveSegment::Line {
                        to: P::from_xy(p1.x, p1.y),
                    },
                ))
            }
            Self::Quad(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                let p0 = segment.control_points[0];
                let p1 = segment.control_points[1];
                let p2 = segment.control_points[2];
                Some((
                    P::from_xy(p0.x, p0.y),
                    CurveSegment::Quad {
                        ctrl: P::from_xy(p1.x, p1.y),
                        to: P::from_xy(p2.x, p2.y),
                    },
                ))
            }
            Self::Cubic(segment) => {
                let segment = segment.range(range.t0, range.t1)?;
                let p0 = segment.control_points[0];
                let p1 = segment.control_points[1];
                let p2 = segment.control_points[2];
                let p3 = segment.control_points[3];
                Some((
                    P::from_xy(p0.x, p0.y),
                    CurveSegment::Cubic {
                        ctrl0: P::from_xy(p1.x, p1.y),
                        ctrl1: P::from_xy(p2.x, p2.y),
                        to: P::from_xy(p3.x, p3.y),
                    },
                ))
            }
        }
    }
}

trait SegmentRangeExtract<T: FloatNumber>: SplitAt<T, Output = [Self; 2]> + Reversed + Copy + Sized {
    fn range(&self, t0: SegmentParam<T>, t1: SegmentParam<T>) -> Option<Self> {
        if t0 == t1 {
            return None;
        }

        if t0.value() < t1.value() {
            Some(self.forward_range(t0, t1))
        } else {
            Some(self.forward_range(t1, t0).reversed())
        }
    }

    fn forward_range(&self, t0: SegmentParam<T>, t1: SegmentParam<T>) -> Self {
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

        let t0 = t0.value();
        let local_t = (t1.value() - t0) / (T::ONE - t0);
        let [segment, _] = right.split_at(local_t);
        segment
    }
}

impl<S, T> SegmentRangeExtract<T> for S
where
    S: SplitAt<T, Output = [S; 2]> + Reversed + Copy + Sized,
    T: FloatNumber,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::circular_merge_list::Merge;
    use crate::kernel::curve::line::LineSegment;
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

    fn line_segments(count: usize) -> Vec<ShapeSegment<f64>> {
        (0..count)
            .map(|_| ShapeSegment {
                segment: Segment::Line(LineSegment {
                    control_points: [[0.0, 0.0].into(), [1.0, 0.0].into()],
                }),
                shape_type: ShapeType::Subject,
            })
            .collect()
    }

    fn segment(segment: Segment<f64>) -> ShapeSegment<f64> {
        ShapeSegment {
            segment,
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
            FloatPointAdapter::<_, i32>::with_scale(FloatRect::new(-10.0, 10.0, -10.0, 10.0), 1000.0);

        let ranges = merge_segment_sets(sets, &segments, &adapter, &mut merge_list);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].segment_index, 2);
        assert_eq!(ranges[0].t0, SegmentParam::Start);
        assert_eq!(ranges[0].t1, SegmentParam::End);
    }

    #[test]
    fn geometry_merge_keeps_expanded_line_alternatives() {
        let adapter =
            FloatPointAdapter::<_, i32>::with_scale(FloatRect::new(-10.0, 10.0, -10.0, 10.0), 1000.0);
        let segments = Vec::from([
            segment(Segment::Line(LineSegment {
                control_points: [[0.0, 0.0].into(), [2.0, 0.0].into()],
            })),
            segment(Segment::Line(LineSegment {
                control_points: [[0.0, 0.0].into(), [2.0, 0.0].into()],
            })),
        ]);
        let mut left = data(Vec::from([range(0, 0.0, 0.5)]), 0, 1000);
        let mut right = data(Vec::from([range(1, 0.5, 1.0)]), 1000, 2000);

        assert!(left.merge(&mut right, &segments, &adapter));

        assert_eq!(left.start, IntPoint::new(0, 0));
        assert_eq!(left.end, IntPoint::new(2000, 0));
        assert_eq!(left.ranges, Vec::from([range(0, 0.0, 1.0), range(1, 0.0, 1.0)]));
    }
}
