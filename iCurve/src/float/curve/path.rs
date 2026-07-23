use crate::float::curve::segment::CurveSegment;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

#[derive(Clone, PartialEq)]
pub struct CurvePath<P: FloatPointCompatible> {
    pub(crate) start: P,
    pub(crate) segments: Vec<CurveSegment<P>>,
}

impl<P: FloatPointCompatible> CurvePath<P> {
    #[inline]
    pub fn start(&self) -> P {
        self.start
    }

    #[inline]
    pub fn segments(&self) -> &[CurveSegment<P>] {
        &self.segments
    }

    #[inline]
    pub fn end_point(&self) -> Option<P> {
        self.segments.last().map(CurveSegment::end_point)
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.end_point()
            .is_some_and(|end| end.x() == self.start.x() && end.y() == self.start.y())
    }
}
