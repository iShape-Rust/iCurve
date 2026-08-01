//! Float-coordinate curve construction, conversion, and Boolean operations.
//!
//! Use [`CurveBuilder`] to create validated closed curves, [`CurveResource`]
//! to pass paths or shape collections to an operation, and
//! [`FloatCurveOverlay`] when conversion scale or solver settings must be
//! controlled explicitly.

mod curve;
mod overlay;
pub mod resource;

/// Ellipses and elliptic-arc representations in float coordinates.
pub mod arc {
    pub use super::curve::arc::{Ellipse, EllipticArc, EllipticArcError, RationalArc, RationalArcError};
}

pub use curve::builder::{CurveBuilder, CurveError as CurveBuildError};
pub use curve::converter::{CurveConversionError, CurveConverter};
pub use curve::path::CurvePath;
pub use curve::segment::CurveSegment;
pub use curve::shape::CurveShape;
pub use i_overlay::i_float::adapter::FloatPointAdapter;
pub use i_overlay::i_float::float::compatible::FloatPointCompatible;
pub use overlay::{
    CurveResourceOverlayExt, FloatCurveOverlay, FloatCurveOverlayOptions, FloatCurveOverlayOptionsError,
};
pub use resource::CurveResource;
