//! Float-coordinate curve construction, conversion, and Boolean operations.
//!
//! Use [`CurveBuilder`] to create validated closed curves, [`CurveResource`]
//! to pass paths or shape collections to an operation, and
//! [`FloatCurveOverlay`] when conversion scale or solver settings must be
//! controlled explicitly.
//!
//! Callers must supply valid geometry with finite coordinates whose absolute
//! values are at most `2^60` for `f32` or `2^500` for `f64`. Computed bounds
//! must also satisfy this range. Bounds computation checks rectangles rather
//! than every input point; its errors do not provide exhaustive input validation.

mod curve;
mod math;
mod overlay;
mod resource;

/// Ellipses and elliptic-arc representations in float coordinates.
pub mod arc {
    pub use super::curve::arc::{Ellipse, EllipticArc, EllipticArcError, RationalArc, RationalArcError};
}

pub use curve::builder::{CurveBuilder, CurveError as CurveBuildError};
pub use curve::converter::{
    CurveConversionError, CurveConversionReport, CurveConverter, CurveToFloatError,
    try_convert_shape_to_float,
};
pub use curve::path::CurvePath;
pub use curve::segment::CurveSegment;
pub use curve::shape::CurveShape;
pub use i_overlay::i_float::adapter::FloatPointAdapter;
pub use i_overlay::i_float::float::compatible::FloatPointCompatible;
pub use i_overlay::i_float::float::rect::FloatRectError;
pub use overlay::{
    CurveResourceOverlayExt, FloatCurveOverlay, FloatCurveOverlayConversionReport, FloatCurveOverlayOptions,
    FloatCurveOverlayOptionsError,
};
pub use resource::CurveResource;
