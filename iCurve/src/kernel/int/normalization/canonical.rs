use crate::kernel::int::curve::segment::Segment;
use crate::kernel::int::normalization::cubic::CubicSShapeNormalization;
use crate::kernel::int::normalization::monotone::decomposition::DecomposeIntoMonotone;
use alloc::vec::Vec;
use i_overlay::i_float::int::number::int::IntNumber;

pub(crate) trait PushCanonicalSegment<I: IntNumber> {
    fn push_canonical(&mut self, segment: Segment<I>);
}

trait PushSegment<I: IntNumber> {
    fn push_normalized(&mut self, segment: Segment<I>);
}

impl<I: IntNumber> PushCanonicalSegment<I> for Vec<Segment<I>> {
    fn push_canonical(&mut self, segment: Segment<I>) {
        match segment {
            Segment::Line(line) => {
                if let Some(s) = line.try_segment() {
                    self.push(s)
                }
            }
            Segment::Quad(quad) => {
                if let Some(s) = quad.try_segment() {
                    self.push_normalized(s);
                }
            }
            Segment::Cubic(cubic) => {
                let segments = cubic.try_segment();
                for s in segments.into_iter() {
                    self.push_normalized(s);
                }
            }
        }
    }
}

impl<I: IntNumber> PushSegment<I> for Vec<Segment<I>> {
    fn push_normalized(&mut self, segment: Segment<I>) {
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
