use i_curve::bool::scale::FixedScaleCurveOverlay;
use i_curve::curve::builder::{CurveError, CurveShapeBuilder};
use i_curve::curve::shape::CurveShape;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;

#[test]
fn subject_roundish_cubic_shape_with_fixed_scale() -> Result<(), CurveError> {
    let subj = roundish_cubic_shape()?;
    let clip: Vec<CurveShape<[f64; 2]>> = Vec::new();

    let result = subj
        .overlay_with_fixed_scale(&clip, OverlayRule::Subject, FillRule::NonZero, 1000.0)
        .expect("fixed scale overlay must run");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].contours.len(), 1);
    assert!(!result[0].contours[0].segments.is_empty());

    Ok(())
}

fn roundish_cubic_shape() -> Result<CurveShape<[f64; 2]>, CurveError> {
    CurveShapeBuilder::new()
        .move_to([-1.0, 0.0])?
        .cubic_to([-1.0, -0.5], [-0.5, -1.0], [0.0, -1.0])?
        .cubic_to([0.5, -1.0], [1.0, -0.5], [1.0, 0.0])?
        .cubic_to([1.0, 0.5], [0.5, 1.0], [0.0, 1.0])?
        .cubic_to([-0.5, 1.0], [-1.0, 0.5], [-1.0, 0.0])?
        .build()
}
