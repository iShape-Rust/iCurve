#![no_std]
#![doc = include_str!("../../README.md")]

extern crate alloc;

mod collections;
pub mod float;
pub mod int;
mod kernel;

pub use float::{
    CurveBuildError, CurveBuilder, CurveConversionError, CurveConverter, CurvePath as FloatCurvePath,
    CurveSegment as FloatCurveSegment, CurveShape as FloatCurveShape, FloatCurveOverlay,
    SingleFloatCurveOverlay,
};
pub use i_overlay::core::fill_rule::FillRule;
pub use i_overlay::core::overlay::ShapeType;
pub use i_overlay::core::overlay_rule::OverlayRule;
pub use i_overlay::core::solver::{Precision, Solver};
pub use i_overlay::i_float::adapter::FloatPointAdapter;
pub use i_overlay::i_float::float::compatible::FloatPointCompatible;
pub use i_overlay::i_float::int::number::int::IntNumber;
pub use i_overlay::i_shape::int::IntPoint;
pub use int::{
    CurveInputError, CurveOverlayOptions, CurvePath as IntCurvePath, CurveSegment as IntCurveSegment,
    CurveShape as IntCurveShape, IntCurveOverlay, overlay,
};
