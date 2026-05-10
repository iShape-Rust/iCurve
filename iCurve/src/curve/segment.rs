use crate::curve::arc::EllipticArc;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

pub enum CurveSegment<P: FloatPointCompatible> {
    Line { to: P },
    Quad { ctrl: P, to: P },
    Cubic { ctrl0: P, ctrl1: P, to: P },
    Arc { arc: EllipticArc<P> },
}
