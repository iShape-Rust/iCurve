use i_curve::int::{CurvePath, CurveSegment, CurveShape, overlay};
use i_curve::{
    CurveBuilder, CurveInputError, CurveOverlayOptions, CurveResource, FillRule, FloatCurveOverlay,
    FloatCurveOverlayOptions, IntCurveOverlay, IntPoint, OverlayRule, Precision, Solver,
};

fn rectangle(x0: i32, y0: i32, x1: i32, y1: i32) -> CurveShape<i32> {
    let start = IntPoint::new(x0, y0);
    CurveShape::from_path(CurvePath::new(
        start,
        vec![
            CurveSegment::Line {
                to: IntPoint::new(x1, y0),
            },
            CurveSegment::Line {
                to: IntPoint::new(x1, y1),
            },
            CurveSegment::Line {
                to: IntPoint::new(x0, y1),
            },
            CurveSegment::Line { to: start },
        ],
    ))
}

#[test]
fn scoped_integer_overlay_is_complete() -> Result<(), CurveInputError> {
    let result = overlay(
        rectangle(0, 0, 100, 100),
        rectangle(50, 25, 150, 75),
        OverlayRule::Intersect,
        FillRule::NonZero,
    )?;

    assert_eq!(result.len(), 1);
    assert!(result[0].contours.iter().all(CurvePath::is_closed));
    Ok(())
}

#[test]
fn extended_builder_validates_before_adding_input() {
    let mut curves = IntCurveOverlay::new();
    assert_eq!(
        curves.add_subject(CurveShape::new(vec![])),
        Err(CurveInputError::EmptyShape)
    );

    let invalid = CurveShape::from_path(CurvePath::new(
        IntPoint::new(0_i32, 0),
        vec![CurveSegment::Line {
            to: IntPoint::new(10, 0),
        }],
    ));
    assert_eq!(
        curves.add_subject(invalid),
        Err(CurveInputError::UnclosedContour { contour: 0 })
    );

    let disconnected = CurveShape::from_path(CurvePath::new(
        IntPoint::new(1_i32, 1),
        vec![CurveSegment::Arc {
            arc: Default::default(),
        }],
    ));
    assert_eq!(
        curves.add_subject(disconnected),
        Err(CurveInputError::DisconnectedArc {
            contour: 0,
            segment: 0,
        })
    );

    curves.add_subject(rectangle(0, 0, 10, 10)).unwrap();
    let result = curves.overlay(OverlayRule::Subject, FillRule::NonZero);
    assert_eq!(result.len(), 1);
}

#[test]
fn options_are_configured_without_public_overlay_fields() {
    let options = CurveOverlayOptions {
        min_chord_length_power: 5,
        angle_tolerance_power: 4,
        max_approximation_depth: 12,
    };
    let curves = IntCurveOverlay::<i32>::new().with_options(options);

    assert_eq!(curves.options(), options);
}

#[test]
fn float_builder_and_converter_are_available_at_top_level() {
    let source = CurveBuilder::new()
        .move_to([0.0_f64, 0.0])
        .unwrap()
        .quad_to([5.0, -2.0], [10.0, 0.0])
        .unwrap()
        .close_contour()
        .unwrap()
        .build()
        .unwrap();

    let converter = i_curve::CurveConverter::<_, i32>::new(source);
    assert!(converter.shape().contours[0].is_closed());

    let _: i_curve::int::arc::RationalArc<i32> = Default::default();
}

fn float_rectangle(x0: f64, y0: f64, x1: f64, y1: f64) -> i_curve::FloatCurveShape<[f64; 2]> {
    CurveBuilder::new()
        .move_to([x0, y0])
        .unwrap()
        .line_to([x1, y0])
        .unwrap()
        .line_to([x1, y1])
        .unwrap()
        .line_to([x0, y1])
        .unwrap()
        .close_contour()
        .unwrap()
        .build()
        .unwrap()
}

#[test]
fn top_level_float_overlay_hides_integer_conversion() {
    let subject = float_rectangle(0.0, 0.0, 10.0, 10.0);
    let clip = float_rectangle(4.0, 2.0, 12.0, 8.0);

    let result = subject.overlay(&clip, OverlayRule::Difference, FillRule::NonZero);
    let wide_result = subject.overlay_as::<i64>(&clip, OverlayRule::Intersect, FillRule::NonZero);
    let path_result = subject.contours()[0].overlay(&clip, OverlayRule::Intersect, FillRule::NonZero);

    let _: &[i_curve::FloatCurvePath<[f64; 2]>] = result[0].contours();
    assert_eq!(wide_result.len(), 1);
    assert_eq!(path_result.len(), 1);
    assert!(
        result
            .iter()
            .flat_map(|shape| shape.contours())
            .all(|path| path.is_closed())
    );
}

#[test]
fn float_curve_resources_accept_shape_collections_and_paths() {
    use i_curve::SingleFloatCurveOverlay as _;

    let subjects = [
        float_rectangle(0.0, 0.0, 2.0, 2.0),
        float_rectangle(4.0, 0.0, 6.0, 2.0),
    ];
    let clip = float_rectangle(1.0, -1.0, 5.0, 3.0);
    let clip_path = &clip.contours()[0];

    assert_eq!(subjects.iter_paths().count(), 2);
    assert_eq!(clip_path.iter_paths().count(), 1);

    let result = subjects
        .as_slice()
        .overlay(clip_path, OverlayRule::Intersect, FillRule::NonZero);
    assert_eq!(result.len(), 2);

    let result = FloatCurveOverlay::<_, i32>::new(&subjects, clip_path)
        .overlay(OverlayRule::Intersect, FillRule::NonZero);
    assert_eq!(result.len(), 2);

    let empty: [i_curve::FloatCurveShape<[f64; 2]>; 0] = [];
    let result =
        FloatCurveOverlay::<_, i32>::new(&empty, &clip).overlay(OverlayRule::Clip, FillRule::NonZero);
    assert_eq!(result.len(), 1);
}

#[test]
fn float_overlay_supports_explicit_i64_solver() {
    let subject = float_rectangle(0.0, 0.0, 10.0, 10.0);
    let clip = float_rectangle(5.0, 2.0, 12.0, 8.0);

    let overlay = FloatCurveOverlay::<_, i64>::new(&subject, &clip)
        .try_with_options(FloatCurveOverlayOptions {
            min_chord_length: Some(0.001),
            ..Default::default()
        })
        .unwrap()
        .with_solver(Solver::with_precision(Precision::MEDIUM));

    assert!(overlay.scale().is_finite());
    assert!(overlay.scale() > 0.0);

    let result = overlay.overlay(OverlayRule::Intersect, FillRule::NonZero);

    assert_eq!(result.len(), 1);
}

#[test]
fn float_results_support_debug_and_container_conversions() {
    let shape = float_rectangle(0.0, 0.0, 10.0, 10.0);

    let debug = format!("{shape:?}");
    assert!(debug.contains("CurveShape"));
    assert_eq!(shape.len(), shape.contours().len());
    assert!(!shape.is_empty());
    let _: &[i_curve::FloatCurvePath<[f64; 2]>] = shape.as_ref();

    let borrowed_segment_count = (&shape).into_iter().flat_map(IntoIterator::into_iter).count();
    assert_eq!(borrowed_segment_count, shape.segment_count());

    let taken_contours = shape.clone().into_contours();
    assert_eq!(taken_contours.len(), shape.len());

    let mut contours: Vec<_> = shape.into_iter().collect();
    let contour = contours.remove(0);
    assert_eq!(contour.len(), borrowed_segment_count);
    assert!(!contour.is_empty());
    let _: &[i_curve::FloatCurveSegment<[f64; 2]>] = contour.as_ref();

    let owned_segments: Vec<_> = contour.clone().into_iter().collect();
    assert_eq!(owned_segments.len(), borrowed_segment_count);

    let expected_start = contour.start();
    let (start, segments) = contour.into_parts();
    assert_eq!(start, expected_start);
    assert_eq!(segments.len(), borrowed_segment_count);
}
