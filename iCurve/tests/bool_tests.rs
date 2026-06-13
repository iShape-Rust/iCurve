use i_curve::bool::overlay::CurveOverlay;
use i_curve::bool::scale::FixedScaleCurveOverlay;
use i_curve::curve::builder::{CurveError, CurveBuilder};
use i_curve::curve::shape::CurveShape;
use i_curve::flatten::approx::LineApproximation;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_shape::float::area::Area;

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

    let expected_area = subj.approximate_to_shape(approximation()).area().abs();
    let result_area = result
        .iter()
        .map(|shape| shape.approximate_to_shape(approximation()).area())
        .sum::<f64>()
        .abs();

    assert!(
        (expected_area - result_area).abs() < 0.001,
        "expected area {expected_area}, got {result_area}"
    );

    Ok(())
}

#[test]
fn union_of_overlapping_polygons_with_near_horizontal_edge() -> Result<(), CurveError> {
    let subject = vec![
        CurveBuilder::new()
            .move_to([-210.0_f32, -130.0])?
            .line_to([70.0, -130.0])?
            .line_to([70.0, 130.0])?
            .line_to([-220.0, 141.000_02])?
            .line_to([-210.0, -130.0])?
            .build()?,
    ];
    let clip = CurveShape { contours: vec![] };

    let mut overlay =
        CurveOverlay::<[f32; 2], i32>::with_subj_and_clip_fixed_scale(&subject, &clip, 100_000.0).unwrap();
    let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);

    assert!(!result.is_empty());
    assert!(result.iter().any(|shape| !shape.contours.is_empty()));

    Ok(())
}

fn approximation() -> LineApproximation<f64> {
    LineApproximation {
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
        .build()
}
