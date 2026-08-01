//! Advanced integer-coordinate curve API.
//!
//! Integer paths and shapes are plain structural containers. [`IntCurveOverlay`]
//! validates that inputs are non-empty, closed, and have connected arcs when
//! they are added. Most applications should prefer the crate-level float API,
//! which selects and applies an integer conversion scale automatically.

mod bool;
mod curve;

/// Fixed-point rational arcs and their supporting ellipse representation.
pub mod arc {
    pub use crate::kernel::int::curve::arc::{
        ArcDirection, ArcPhase, ArcSegment as RationalArc, ArcVector, EllipseFrame,
    };
    pub use crate::kernel::int::curve::param::SegmentParam as CurveParameter;
}

pub use bool::overlay::{
    CurveInputError, CurveOverlayOptions, CurveOverlayOptionsError, IntCurveOverlay, overlay,
};
pub use curve::path::CurvePath;
pub use curve::segment::CurveSegment;
pub use curve::shape::CurveShape;
/// Integer engine supported by curve conversion and Boolean operations.
///
/// This trait is sealed and implemented for `i16`, `i32`, and `i64`.
pub use i_overlay::core::integer::OverlayInt as CurveInt;
pub use i_overlay::core::overlay::ShapeType;
pub use i_overlay::i_float::int::number::int::IntNumber;
pub use i_overlay::i_shape::int::IntPoint;

/// Bits reserved for intermediate polynomial coefficient growth.
pub(crate) const CURVE_COORDINATE_SAFETY_BITS: u32 = 6;
