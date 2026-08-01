mod curve;
mod overlay;
pub mod resource;

pub mod arc {
    pub use super::curve::arc::{Ellipse, EllipticArc, EllipticArcError, RationalArc, RationalArcError};
}

pub use curve::builder::{CurveBuilder, CurveError as CurveBuildError};
pub use curve::converter::{CurveConversionError, CurveConverter};
pub use curve::path::CurvePath;
pub use curve::segment::CurveSegment;
pub use curve::shape::CurveShape;
pub use overlay::{FloatCurveOverlay, SingleFloatCurveOverlay};
pub use resource::CurveResource;
