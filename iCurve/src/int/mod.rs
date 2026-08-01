mod bool;
mod curve;

pub mod arc {
    pub use crate::kernel::int::curve::arc::{
        ArcDirection, ArcPhase, ArcSegment as RationalArc, ArcVector, EllipseFrame,
    };
    pub use crate::kernel::int::curve::param::SegmentParam as CurveParameter;
}

pub use bool::overlay::{CurveInputError, CurveOverlayOptions, IntCurveOverlay, overlay};
pub use curve::path::CurvePath;
pub use curve::segment::CurveSegment;
pub use curve::shape::CurveShape;

/// Bits reserved for intermediate polynomial coefficient growth.
pub(crate) const CURVE_COORDINATE_SAFETY_BITS: u32 = 6;
