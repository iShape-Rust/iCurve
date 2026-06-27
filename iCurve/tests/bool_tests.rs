use i_curve::bool::overlay::CurveOverlay;
use i_curve::curve::builder::{CurveBuilder, CurveError};
use i_curve::curve::shape::CurveShape;
use i_curve::flatten::approximation::LineApproximation;
use i_curve::flatten::condition::FlatParams;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_shape::float::area::Area;


fn approximation() -> FlatParams<f64> {
    FlatParams {
        min_cos: 0.999,
        min_segment_sqr_length: 0.000_001,
    }
}

fn roundish_cubic_shape() -> Result<CurveShape<[f64; 2]>, CurveError> {
    CurveBuilder::new()
        .move_to([-1.0, 0.0])?
        .cubic_to([-1.0, -0.5], [-0.5, -1.0], [0.0, -1.0])?
        .cubic_to([0.5, -1.0], [1.0, -0.5], [1.0, 0.0])?
        .cubic_to([1.0, 0.5], [0.5, 1.0], [0.0, 1.0])?
        .cubic_to([-0.5, 1.0], [-1.0, 0.5], [-1.0, 0.0])?
        .build_shape()
}
