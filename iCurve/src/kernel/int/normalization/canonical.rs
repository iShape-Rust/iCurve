use crate::kernel::int::curve::segment::Segment;
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
