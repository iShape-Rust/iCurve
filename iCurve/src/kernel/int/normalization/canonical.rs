use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::{SegmentParam, interpolate_segment_param};
use crate::kernel::int::curve::point_at::PointAt;
use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::curve::split_at::{SplitAt, segment_range};
use crate::kernel::int::normalization::cubic::CubicSShapeNormalization;
use crate::kernel::int::normalization::monotone::decomposition::DecomposeIntoMonotone;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;

pub(crate) trait PushCanonicalSegment<I: IntNumber> {
    fn push_canonical(&mut self, segment: Segment<I>);
}

pub(crate) trait PushSimpleSegment<I: IntNumber> {
    fn push_simple(&mut self, segment: Segment<I>);
}

pub(crate) trait PushCanonicalSimpleSegment<I: IntNumber> {
    fn push_canonical_simple(&mut self, segment: Segment<I>);
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParametricSegment<I: IntNumber> {
    pub(crate) curve: Segment<I>,
    pub(crate) start: SegmentParam<I>,
    pub(crate) end: SegmentParam<I>,
}

pub(crate) trait PushCanonicalSimpleParametricSegment<I: IntNumber> {
    fn push_canonical_simple_parametric(&mut self, segment: Segment<I>);
}

impl<I: IntNumber> PushCanonicalSegment<I> for Vec<Segment<I>> {
    fn push_canonical(&mut self, segment: Segment<I>) {
        let mut simple_segments = Vec::new();
        simple_segments.push_simple(segment);

        for simple in simple_segments {
            self.push_canonical_simple(simple);
        }
    }
}

impl<I: IntNumber> PushSimpleSegment<I> for Vec<Segment<I>> {
    fn push_simple(&mut self, segment: Segment<I>) {
        match segment {
            Segment::Line(line) => {
                if let Some(s) = line.try_segment() {
                    self.push(s)
                }
            }
            Segment::Quad(quad) => {
                for piece in quad.split_at_cusp() {
                    if let Some(simple) = piece.try_segment() {
                        self.push(simple);
                    }
                }
            }
            Segment::Cubic(cubic) => {
                let segments = cubic.try_segment();
                for segment in segments {
                    self.push_simple_without_self_intersection(segment);
                }
            }
        }
    }
}

trait PushSimpleWithoutSelfIntersection<I: IntNumber> {
    fn push_simple_without_self_intersection(&mut self, segment: Segment<I>);
}

impl<I: IntNumber> PushSimpleWithoutSelfIntersection<I> for Vec<Segment<I>> {
    fn push_simple_without_self_intersection(&mut self, segment: Segment<I>) {
        match segment {
            Segment::Line(line) => self.push(Segment::Line(line)),
            Segment::Quad(quad) => {
                for piece in quad.split_at_cusp() {
                    if let Some(simple) = piece.try_segment() {
                        self.push(simple);
                    }
                }
            }
            Segment::Cubic(cubic) => {
                for piece in cubic.split_at_cusps() {
                    if let Some(simple) = piece.try_cubic_without_self_intersection() {
                        self.push(simple);
                    }
                }
            }
        }
    }
}

impl<I: IntNumber> PushCanonicalSimpleSegment<I> for Vec<Segment<I>> {
    fn push_canonical_simple(&mut self, segment: Segment<I>) {
        match segment {
            Segment::Line(line) => self.push(Segment::Line(line)),
            Segment::Quad(quad) => {
                for ms in quad.decompose_into_monotone().into_iter() {
                    self.push(Segment::Quad(ms));
                }
            }
            Segment::Cubic(cubic) => {
                for ms in cubic.decompose_into_monotone().into_iter() {
                    match ms.normalize_monotone_without_s_shape() {
                        CubicSShapeNormalization::NoS(s) => {
                            self.push(Segment::Cubic(s));
                        }
                        CubicSShapeNormalization::Pieces([s0, s1]) => {
                            self.push(Segment::Cubic(s0));
                            self.push(Segment::Cubic(s1));
                        }
                    }
                }
            }
        }
    }
}

impl<I: IntNumber> PushCanonicalSimpleParametricSegment<I> for Vec<ParametricSegment<I>> {
    fn push_canonical_simple_parametric(&mut self, segment: Segment<I>) {
        let zero = SegmentParam::new(I::ZERO);
        let one = SegmentParam::new(I::from_wide(SegmentParam::<I>::DENOMINATOR));

        match segment {
            Segment::Line(line) => self.push(ParametricSegment {
                curve: Segment::Line(line),
                start: zero,
                end: one,
            }),
            Segment::Quad(quad) => {
                let roots = quad.monotone_roots();
                let mut start = zero;
                let mut start_point = quad.chord().a;

                for end in roots.into_iter().chain(core::iter::once(one)) {
                    let end_point = quad.control_points.point_at(end);
                    let curve = Segment::Quad(segment_range(&quad, start, start_point, end, end_point));
                    if !curve.chord().is_zero_length() {
                        self.push(ParametricSegment { curve, start, end });
                    }
                    start = end;
                    start_point = end_point;
                }
            }
            Segment::Cubic(cubic) => {
                let roots = cubic.monotone_roots();
                let mut start = zero;
                let mut start_point = cubic.chord().a;

                for end in roots.into_iter().chain(core::iter::once(one)) {
                    let end_point = cubic.control_points.point_at(end);
                    let monotone = segment_range(&cubic, start, start_point, end, end_point);
                    if monotone.chord().is_zero_length() {
                        start = end;
                        start_point = end_point;
                        continue;
                    }

                    if let Some(local) = monotone.s_shape_split_param() {
                        let [first, last] = monotone.split_at(local);
                        let middle = interpolate_segment_param(start, end, local);
                        self.push(ParametricSegment {
                            curve: Segment::Cubic(first),
                            start,
                            end: middle,
                        });
                        self.push(ParametricSegment {
                            curve: Segment::Cubic(last),
                            start: middle,
                            end,
                        });
                    } else {
                        self.push(ParametricSegment {
                            curve: Segment::Cubic(monotone),
                            start,
                            end,
                        });
                    }

                    start = end;
                    start_point = end_point;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::int::curve::quad::QuadSegment;
    use i_overlay::i_shape::int::IntPoint;

    #[test]
    fn parametric_quad_pieces_share_monotone_root_point() {
        let quad = Segment::Quad(QuadSegment {
            control_points: [
                IntPoint::new(110, 55),
                IntPoint::new(-177, -145),
                IntPoint::new(-110, 55),
            ],
        });
        let mut pieces = Vec::new();

        pieces.push_canonical_simple_parametric(quad);

        assert_eq!(pieces.len(), 3);
        for pair in pieces.windows(2) {
            let [left, right] = pair else { unreachable!() };
            assert_eq!(left.end, right.start);
            assert_eq!(
                left.curve.chord().b,
                right.curve.chord().a,
                "canonical pieces must share the point at parameter {:?}",
                left.end
            );
        }
    }
}
