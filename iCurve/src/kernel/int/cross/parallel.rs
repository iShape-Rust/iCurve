use crate::kernel::int::cross::chord::ChordCross;
use crate::kernel::int::cross::intersector::{ContactPoint, ContactType, SegmentIntersector, global_param};
use crate::kernel::int::curve::chord::{Chord, SegmentChord};
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::segment::Segment;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Clone, Copy)]
enum ProjectionAxis {
    X,
    Y,
}

#[derive(Clone, Copy)]
struct Sample<I: IntNumber> {
    point: IntPoint<I>,
    param: SegmentParam<I>,
}

#[derive(Clone, Copy)]
struct PolylineEdge<I: IntNumber> {
    a: Sample<I>,
    b: Sample<I>,
}

struct ParallelPointIter<I: IntNumber> {
    segment: Segment<I>,
    parts: usize,
    next: usize,
    reversed: bool,
}

struct ParallelEdgeIter<I: IntNumber> {
    points: ParallelPointIter<I>,
    previous: Option<Sample<I>>,
    #[cfg(debug_assertions)]
    last_end: Option<IntPoint<I>>,
}

impl<I: IntNumber> SegmentIntersector<I> {
    pub(super) fn intersect_parallel(
        &self,
        s0: Segment<I>,
        center0: SegmentParam<I>,
        step0: SegmentParam<I>,
        s1: Segment<I>,
        center1: SegmentParam<I>,
        step1: SegmentParam<I>,
        output: &mut Vec<ContactPoint<I>>,
    ) {
        let chord0 = s0.chord();
        let chord1 = s1.chord();
        let axis = ProjectionAxis::for_chords(chord0, chord1);
        let parts0 = chord0.parts_count(self.options.min_len_pow2, self.options.max_parts_count_log);
        let parts1 = chord1.parts_count(self.options.min_len_pow2, self.options.max_parts_count_log);
        let range0 = chord0.projection_range(axis);
        let range1 = chord1.projection_range(axis);

        let mut edges0 = ParallelEdgeIter::new(s0, axis, parts0);
        let mut edges1 = ParallelEdgeIter::new(s1, axis, parts1);
        let mut edge0 = edges0.next();
        let mut edge1 = edges1.next();

        while let (Some(a), Some(b)) = (edge0, edge1) {
            let a0 = axis.value(a.a.point);
            let a1 = axis.value(a.b.point);
            let b0 = axis.value(b.a.point);
            let b1 = axis.value(b.b.point);

            if a1 < b0 {
                edge0 = edges0.next();
                continue;
            }
            if b1 < a0 {
                edge1 = edges1.next();
                continue;
            }

            let chord_a = SegmentChord {
                a: a.a.point,
                b: a.b.point,
            };
            let chord_b = SegmentChord {
                a: b.a.point,
                b: b.b.point,
            };
            match chord_a.cross(&chord_b, self.options.cross_radius) {
                Some(ChordCross::Point(point))
                    if chord_a.vector().cross_product(chord_b.vector()) != I::Wide::ZERO =>
                {
                    push_contact(
                        output,
                        ContactPoint {
                            point,
                            t0: global_param(center0, step0, edge_param(a, point)),
                            t1: global_param(center1, step1, edge_param(b, point)),
                            contact_type: ContactType::Cross,
                        },
                    );
                }
                Some(ChordCross::Point(_)) => {}
                Some(ChordCross::Overlay) => {
                    push_internal_overlap_ends(
                        output, axis, a, b, range0, range1, center0, step0, center1, step1,
                    );
                }
                None => {}
            }

            if a1 <= b1 {
                edge0 = edges0.next();
            }
            if b1 <= a1 {
                edge1 = edges1.next();
            }
        }
    }
}

impl ProjectionAxis {
    fn for_chords<I: IntNumber>(a: SegmentChord<I>, b: SegmentChord<I>) -> Self {
        let av = a.vector();
        let bv = b.vector();
        let x = av.x.unsigned_abs() + bv.x.unsigned_abs();
        let y = av.y.unsigned_abs() + bv.y.unsigned_abs();
        if x >= y { Self::X } else { Self::Y }
    }

    #[inline]
    fn value<I: IntNumber>(self, point: IntPoint<I>) -> I {
        match self {
            Self::X => point.x,
            Self::Y => point.y,
        }
    }
}

impl<I: IntNumber> ParallelPointIter<I> {
    fn new(segment: Segment<I>, axis: ProjectionAxis, parts: usize) -> Self {
        let chord = segment.chord();
        Self {
            segment,
            parts,
            next: 0,
            reversed: axis.value(chord.a) > axis.value(chord.b),
        }
    }
}

impl<I: IntNumber> Iterator for ParallelPointIter<I> {
    type Item = Sample<I>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.parts {
            return None;
        }
        let index = if self.reversed {
            self.parts - self.next
        } else {
            self.next
        };
        self.next += 1;

        let denominator = SegmentParam::<I>::DENOMINATOR;
        let value = denominator * I::Wide::from_usize(index) / I::Wide::from_usize(self.parts);
        let param = SegmentParam::new(I::from_wide(value));
        Some(Sample {
            point: segment_point_at(self.segment, param),
            param,
        })
    }
}

impl<I: IntNumber> ParallelEdgeIter<I> {
    fn new(segment: Segment<I>, axis: ProjectionAxis, parts: usize) -> Self {
        Self {
            points: ParallelPointIter::new(segment, axis, parts),
            previous: None,
            #[cfg(debug_assertions)]
            last_end: None,
        }
    }
}

impl<I: IntNumber> Iterator for ParallelEdgeIter<I> {
    type Item = PolylineEdge<I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let sample = self.points.next()?;
            if let Some(previous) = self.previous.replace(sample) {
                if previous.point != sample.point {
                    #[cfg(debug_assertions)]
                    {
                        debug_assert!(
                            self.last_end.is_none_or(|end| end == previous.point),
                            "consecutive ParallelEdgeIter edges must share an endpoint"
                        );
                        self.last_end = Some(sample.point);
                    }
                    return Some(PolylineEdge {
                        a: previous,
                        b: sample,
                    });
                }
            }
        }
    }
}

impl<I: IntNumber> SegmentChord<I> {
    fn projection_range(&self, axis: ProjectionAxis) -> (I, I) {
        let a = axis.value(self.a);
        let b = axis.value(self.b);
        if a <= b { (a, b) } else { (b, a) }
    }

    /// Returns a power-of-two subdivision count for an approximate target piece
    /// length of `2^min_len_pow2`. This is a preferred scale, not a strict
    /// upper bound for every generated curve piece.
    fn parts_count(&self, min_len_pow2: u32, max_parts_count_log: u32) -> usize {
        let len_log = self.vector().sqr_length().ilog2() >> 1;
        if len_log <= min_len_pow2 {
            1
        } else {
            let cnt_log = (len_log - min_len_pow2).min(max_parts_count_log);
            1 << cnt_log
        }
    }
}

fn segment_point_at<I: IntNumber>(segment: Segment<I>, param: SegmentParam<I>) -> IntPoint<I> {
    match segment {
        Segment::Line(line) => line.control_points.point_at(param),
        Segment::Quad(quad) => quad.control_points.point_at(param),
        Segment::Cubic(cubic) => cubic.control_points.point_at(param),
        Segment::Arc(arc) => arc.not_implemented("parallel point lookup"),
    }
}

fn edge_param<I: IntNumber>(edge: PolylineEdge<I>, point: IntPoint<I>) -> SegmentParam<I> {
    let chord = SegmentChord {
        a: edge.a.point,
        b: edge.b.point,
    };
    interpolate_param(edge.a.param, edge.b.param, chord.param_for_point(point))
}

fn interpolate_param<I: IntNumber>(
    a: SegmentParam<I>,
    b: SegmentParam<I>,
    local: SegmentParam<I>,
) -> SegmentParam<I> {
    let denominator = SegmentParam::<I>::DENOMINATOR;
    let delta = b.value() - a.value();
    let value = a.value() + delta * local.value() / denominator;
    SegmentParam::new(I::from_wide(value))
}

fn push_internal_overlap_ends<I: IntNumber>(
    output: &mut Vec<ContactPoint<I>>,
    axis: ProjectionAxis,
    a: PolylineEdge<I>,
    b: PolylineEdge<I>,
    range0: (I, I),
    range1: (I, I),
    center0: SegmentParam<I>,
    step0: SegmentParam<I>,
    center1: SegmentParam<I>,
    step1: SegmentParam<I>,
) {
    for sample in [a.a, a.b] {
        let value = axis.value(sample.point);
        if is_unit_end(sample.param) && range1.0 < value && value < range1.1 {
            push_tangent(output, sample.point, a, b, center0, step0, center1, step1);
        }
    }
    for sample in [b.a, b.b] {
        let value = axis.value(sample.point);
        if is_unit_end(sample.param) && range0.0 < value && value < range0.1 {
            push_tangent(output, sample.point, a, b, center0, step0, center1, step1);
        }
    }
}

#[inline]
fn is_unit_end<I: IntNumber>(param: SegmentParam<I>) -> bool {
    param.value() == I::Wide::ZERO || param.value() == SegmentParam::<I>::DENOMINATOR
}

fn push_tangent<I: IntNumber>(
    output: &mut Vec<ContactPoint<I>>,
    point: IntPoint<I>,
    a: PolylineEdge<I>,
    b: PolylineEdge<I>,
    center0: SegmentParam<I>,
    step0: SegmentParam<I>,
    center1: SegmentParam<I>,
    step1: SegmentParam<I>,
) {
    push_contact(
        output,
        ContactPoint {
            point,
            t0: global_param(center0, step0, edge_param(a, point)),
            t1: global_param(center1, step1, edge_param(b, point)),
            contact_type: ContactType::Tangent,
        },
    );
}

fn push_contact<I: IntNumber>(output: &mut Vec<ContactPoint<I>>, contact: ContactPoint<I>) {
    if !output
        .iter()
        .any(|item| item.point == contact.point && item.contact_type == contact.contact_type)
    {
        output.push(contact);
    }
}

#[cfg(test)]
mod tests {
    use super::{ParallelPointIter, ProjectionAxis};
    use crate::kernel::int::cross::intersector::{ContactType, SegmentIntersector, SplitOptions};
    use crate::kernel::int::curve::line::LineSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec::Vec;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn point_iter_sorts_reversed_segment() {
        let segment = Segment::Line(LineSegment {
            control_points: [IntPoint::new(32, 0), IntPoint::new(0, 0)],
        });
        let points: Vec<_> = ParallelPointIter::new(segment, ProjectionAxis::X, 4)
            .map(|sample| sample.point.x)
            .collect();
        assert_eq!(points, [0, 8, 16, 24, 32]);
    }

    #[test]
    fn overlap_returns_internal_ends_as_tangents() {
        let a = Segment::Line(LineSegment {
            control_points: [IntPoint::new(0, 0), IntPoint::new(64, 0)],
        });
        let b = Segment::Line(LineSegment {
            control_points: [IntPoint::new(48, 0), IntPoint::new(16, 0)],
        });
        let contacts = SegmentIntersector::new(a, b, SplitOptions::default()).intersect();

        assert_eq!(contacts.len(), 2);
        assert!(
            contacts
                .iter()
                .all(|contact| contact.contact_type == ContactType::Tangent)
        );
        assert!(
            contacts
                .iter()
                .any(|contact| contact.point == IntPoint::new(16, 0))
        );
        assert!(
            contacts
                .iter()
                .any(|contact| contact.point == IntPoint::new(48, 0))
        );
    }
}
