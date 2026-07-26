use crate::float::curve::arc::RationalArc;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

#[derive(Clone, PartialEq)]
pub enum CurveSegment<P: FloatPointCompatible> {
    Line { to: P },
    Quad { ctrl: P, to: P },
    Cubic { ctrl0: P, ctrl1: P, to: P },
    Arc { arc: RationalArc<P> },
}

impl<P: FloatPointCompatible> CurveSegment<P> {
    #[inline]
    pub fn end_point(&self) -> P {
        match self {
            Self::Line { to } | Self::Quad { to, .. } | Self::Cubic { to, .. } => *to,
            Self::Arc { arc } => arc.end_point(),
        }
    }
}
