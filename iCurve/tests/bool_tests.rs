use i_curve::bool::overlay::CurveOverlay;
use i_curve::curve::builder::{CurveBuilder, CurveError};
use i_curve::curve::shape::CurveShape;
use i_curve::flatten::approx::LineApproximation;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_shape::float::area::Area;

#[test]
fn subject_roundish_cubic_shape_with_fixed_scale() -> Result<(), CurveError> {
    let subj = roundish_cubic_shape()?;
    let clip: Vec<CurveShape<[f64; 2]>> = Vec::new();

    let adapter: FloatPointAdapter<_, i32> = FloatPointAdapter::with_radius_and_scale(1000.0, 1000.0);

    let mut overlay = CurveOverlay::with_adapter(adapter.clone());
    _ = overlay.add_shape(&subj, ShapeType::Subject);
    _ = overlay.add_shape(&clip, ShapeType::Subject);
    let result = overlay.overlay(OverlayRule::Subject, FillRule::NonZero);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].contours.len(), 1);
    assert!(!result[0].contours[0].segments.is_empty());

    let fp_adapter = adapter.to_float_point_adapter();

    let expected_area = subj
        .approximate_with_adapter(approximation(), &fp_adapter)
        .area()
        .abs();
    let result_area = result
        .iter()
        .map(|shape| {
            shape
                .approximate_with_adapter(approximation(), &fp_adapter)
                .area()
        })
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
            .build_shape()?,
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
        .build_shape()
}
