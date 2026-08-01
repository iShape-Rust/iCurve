#![no_std]
#![doc = include_str!("../../README.md")]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

extern crate alloc;

mod collections;
pub mod float;
pub mod int;
mod kernel;

pub use float::{
    CurveBuildError, CurveBuilder, CurveConversionError, CurveConversionReport, CurvePath as FloatCurvePath,
    CurveResource, CurveResourceOverlayExt, CurveSegment as FloatCurveSegment, CurveShape as FloatCurveShape,
    FloatCurveOverlay, FloatCurveOverlayConversionReport, FloatCurveOverlayOptions,
    FloatCurveOverlayOptionsError,
};
pub use i_overlay::core::fill_rule::FillRule;
pub use i_overlay::core::overlay_rule::OverlayRule;
pub use i_overlay::core::solver::{Precision, Solver};
