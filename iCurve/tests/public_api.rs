use i_curve::float::arc::{
    Ellipse as FloatEllipse, EllipticArc as FloatEllipticArc, EllipticArcError,
    RationalArcError as FloatRationalArcError,
};
use i_curve::int::arc::{ArcDirection, ArcPhase, ArcVector, EllipseFrame, RationalArc, RationalArcError};
use i_curve::int::{
    CurveInputError, CurveOverlayOptions, CurveOverlayOptionsError, CurvePath, CurveSegment, CurveShape,
    IntCurveOverlay, IntPoint, overlay,
};
use i_curve::{
    CurveBuilder, CurveConversionReport, FillRule, FloatCurveOverlay, FloatCurveOverlayConversionReport,
    FloatCurveOverlayOptions, OverlayRule, Precision, Solver,
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

fn invalid_arc(control_points: [IntPoint<i32>; 3]) -> RationalArc<i32> {
    RationalArc {
        ellipse: EllipseFrame {
            center: IntPoint::new(0, 0),
            axis_x: ArcVector { x: 0, y: 0 },
            axis_y: ArcVector { x: 0, y: 0 },
        },
        control_points,
        weights: [0; 3],
        start_phase: ArcPhase { cos: 0, sin: 0 },
        end_phase: ArcPhase { cos: 0, sin: 0 },
        direction: ArcDirection::CounterClockwise,
    }
}

#[test]
fn float_arcs_expose_validation() {
    let arc = FloatEllipticArc {
        ellipse: FloatEllipse {
            center: [0.0_f64, 0.0],
            radius_x: 10.0,
            radius_y: 5.0,
            rotation: 0.0,
        },
        start_angle: 0.0,
        sweep_angle: core::f64::consts::FRAC_PI_2,
    };
    assert_eq!(arc.validate(), Ok(()));

    let mut invalid_arc = arc;
    invalid_arc.ellipse.radius_x = 0.0;
    assert_eq!(invalid_arc.validate(), Err(EllipticArcError::NonPositiveRadius));

    let mut rational = arc.to_rational_arcs().unwrap()[0];
    assert_eq!(rational.validate(), Ok(()));
    rational.weights[1] = 0.0;
    assert_eq!(rational.validate(), Err(FloatRationalArcError::NonPositiveWeight));

    rational.weights[1] = 1.0;
    rational.ellipse.radius_y = 0.0;
    let error = rational.validate().unwrap_err();
    assert_eq!(
        error,
        FloatRationalArcError::Elliptic(EllipticArcError::NonPositiveRadius)
    );
    assert!(core::error::Error::source(&error).is_some_and(|source| source.is::<EllipticArcError>()));
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
            arc: invalid_arc([IntPoint::new(0, 0); 3]),
        }],
    ));
    assert_eq!(
        curves.add_subject(disconnected),
        Err(CurveInputError::DisconnectedArc {
            contour: 0,
            segment: 0,
        })
    );

    let start = IntPoint::new(0_i32, 0);
    let invalid = CurveShape::from_path(CurvePath::new(
        start,
        vec![CurveSegment::Arc {
            arc: invalid_arc([start, IntPoint::new(1, 1), start]),
        }],
    ));
    let error = curves.add_subject(invalid).unwrap_err();
    assert_eq!(
        error,
        CurveInputError::InvalidArc {
            contour: 0,
            segment: 0,
            error: RationalArcError::DegenerateEllipse,
        }
    );
    assert!(core::error::Error::source(&error).is_some_and(|source| source.is::<RationalArcError>()));

    curves.add_subject(rectangle(0, 0, 10, 10)).unwrap();
    let result = curves.overlay(OverlayRule::Subject, FillRule::NonZero);
    assert_eq!(result.len(), 1);
}

#[test]
fn options_are_configured_without_public_overlay_fields() {
    let options = CurveOverlayOptions::default()
        .with_min_chord_length_power(5)
        .with_angle_tolerance_power(4)
        .with_max_approximation_depth(12)
        .with_refinement_subdivision_power(2)
        .with_refinement_angle_tolerance_power(6)
        .with_max_refinement_iterations(1);
    let curves = IntCurveOverlay::<i32>::new().try_with_options(options).unwrap();

    assert_eq!(curves.options(), options);
    assert_eq!(CurveOverlayOptions::default().refinement_subdivision_power, 3);
    assert_eq!(CurveOverlayOptions::default().refinement_angle_tolerance_power, 8);
    assert_eq!(CurveOverlayOptions::default().max_refinement_iterations, 2);

    assert!(
        CurveOverlayOptions::default()
            .with_max_approximation_depth(0)
            .validate()
            .is_ok()
    );
    assert!(
        CurveOverlayOptions::default()
            .with_max_approximation_depth(CurveOverlayOptions::MAX_APPROXIMATION_DEPTH)
            .validate()
            .is_ok()
    );

    let error = IntCurveOverlay::<i32>::new()
        .try_with_options(
            CurveOverlayOptions::default()
                .with_max_approximation_depth(CurveOverlayOptions::MAX_APPROXIMATION_DEPTH + 1),
        )
        .err()
        .unwrap();
    match error {
        CurveOverlayOptionsError::MaxApproximationDepthTooLarge { requested, maximum } => {
            assert_eq!(requested, CurveOverlayOptions::MAX_APPROXIMATION_DEPTH + 1);
            assert_eq!(maximum, CurveOverlayOptions::MAX_APPROXIMATION_DEPTH);
        }
        _ => panic!("unexpected options error"),
    }

    assert_eq!(
        CurveOverlayOptions::default()
            .with_refinement_subdivision_power(CurveOverlayOptions::MAX_REFINEMENT_SUBDIVISION_POWER + 1,)
            .validate(),
        Err(CurveOverlayOptionsError::RefinementSubdivisionPowerTooLarge {
            requested: CurveOverlayOptions::MAX_REFINEMENT_SUBDIVISION_POWER + 1,
            maximum: CurveOverlayOptions::MAX_REFINEMENT_SUBDIVISION_POWER,
        })
    );
    assert_eq!(
        CurveOverlayOptions::default()
            .with_max_refinement_iterations(CurveOverlayOptions::MAX_REFINEMENT_ITERATIONS + 1)
            .validate(),
        Err(CurveOverlayOptionsError::MaxRefinementIterationsTooLarge {
            requested: CurveOverlayOptions::MAX_REFINEMENT_ITERATIONS + 1,
            maximum: CurveOverlayOptions::MAX_REFINEMENT_ITERATIONS,
        })
    );

    let float_options = FloatCurveOverlayOptions::<f64>::default()
        .with_refinement_subdivision_power(2)
        .with_refinement_angle_tolerance_power(6)
        .with_max_refinement_iterations(1);
    assert_eq!(float_options.refinement_subdivision_power, 2);
    assert_eq!(float_options.refinement_angle_tolerance_power, 6);
    assert_eq!(float_options.max_refinement_iterations, 1);
}

#[test]
fn float_builder_is_at_top_level_and_converter_is_scoped() {
    let source = CurveBuilder::new()
        .move_to([0.0_f64, 0.0])
        .unwrap()
        .quad_to([5.0, -2.0], [10.0, 0.0])
        .unwrap()
        .close_contour()
        .unwrap()
        .build()
        .unwrap();

    let converter = i_curve::float::CurveConverter::<_, i32>::new(&source);
    let _: &i_curve::float::FloatPointAdapter<[f64; 2], i32> = converter.adapter();
    assert!(converter.scale() > 0.0);
    assert!(converter.shape().contours[0].is_closed());
    assert_eq!(source.contours().len(), 1);
    let _: CurveConversionReport = converter.report();
    let (_, converted, report) = converter.into_parts();
    assert_eq!(converted.contours.len(), 1);
    assert_eq!(report.contour_count, 1);

    fn assert_float_point<P: i_curve::float::FloatPointCompatible>() {}
    assert_float_point::<[f32; 2]>();
    assert_float_point::<[f64; 2]>();

    fn assert_curve_int<I: i_curve::int::CurveInt>() {}
    assert_curve_int::<i16>();
    assert_curve_int::<i32>();
    assert_curve_int::<i64>();

    fn assert_integer_surface<I: i_curve::int::CurveInt>() {
        let _: Option<i_curve::int::CurvePath<I>> = None;
        let _: Option<i_curve::int::CurveSegment<I>> = None;
        let _: Option<i_curve::int::CurveShape<I>> = None;
        let _: Option<i_curve::int::arc::ArcPhase<I>> = None;
        let _: Option<i_curve::int::arc::ArcVector<I>> = None;
        let _: Option<i_curve::int::arc::EllipseFrame<I>> = None;
        let _: Option<i_curve::int::arc::RationalArc<I>> = None;
    }
    assert_integer_surface::<i16>();
    assert_integer_surface::<i32>();
    assert_integer_surface::<i64>();

    assert_eq!(
        invalid_arc([IntPoint::new(0, 0); 3]).validate(),
        Err(RationalArcError::DegenerateEllipse)
    );
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
            .all(|path| !path.segments().is_empty())
    );
}

#[test]
fn float_curve_resources_accept_shape_collections_and_paths() {
    use i_curve::CurveResourceOverlayExt as _;

    let subjects = [
        float_rectangle(0.0, 0.0, 2.0, 2.0),
        float_rectangle(4.0, 0.0, 6.0, 2.0),
    ];
    let clip = float_rectangle(1.0, -1.0, 5.0, 3.0);
    let clip_path = &clip.contours()[0];

    let converted = i_curve::float::CurveConverter::<_, i32>::new(&subjects);
    assert_eq!(converted.shape().contours.len(), 2);
    let converted = i_curve::float::CurveConverter::<_, i32>::new(clip_path);
    assert_eq!(converted.shape().contours.len(), 1);

    let result = subjects
        .as_slice()
        .overlay(clip_path, OverlayRule::Intersect, FillRule::NonZero);
    assert_eq!(result.len(), 2);

    let subject_refs = vec![&subjects[0], &subjects[1]];
    let result = subject_refs.overlay(clip_path, OverlayRule::Intersect, FillRule::NonZero);
    assert_eq!(result.len(), 2);

    let boxed_subject = Box::new(subjects[0].clone());
    let result = boxed_subject.overlay(clip_path, OverlayRule::Intersect, FillRule::NonZero);
    assert_eq!(result.len(), 1);

    let result = FloatCurveOverlay::<_, i32>::new(&subjects, clip_path)
        .overlay(OverlayRule::Intersect, FillRule::NonZero);
    assert_eq!(result.len(), 2);

    let empty: [i_curve::FloatCurveShape<[f64; 2]>; 0] = [];
    let converted = i_curve::float::CurveConverter::<_, i32>::new(&empty);
    assert!(converted.shape().contours.is_empty());
    let overlay = FloatCurveOverlay::<_, i32>::new(&empty, &clip);
    let report: FloatCurveOverlayConversionReport = overlay.conversion_report();
    assert_eq!(report.subject.contour_count, 0);
    assert_eq!(report.clip.expect("clip report").contour_count, 1);
    let result = overlay.overlay(OverlayRule::Clip, FillRule::NonZero);
    assert_eq!(result.len(), 1);
}

#[test]
fn manual_integer_overlay_round_trips_through_one_adapter() {
    let subject = float_rectangle(0.0, 0.0, 10.0, 10.0);
    let clip = float_rectangle(4.0, 2.0, 12.0, 8.0);
    let combined = [&subject, &clip];
    let adapter = i_curve::float::CurveConverter::<_, i64>::new(&combined)
        .adapter()
        .clone();
    let integer_subject = i_curve::float::CurveConverter::<_, i64>::try_with_adapter(&subject, &adapter)
        .unwrap()
        .into_shape();
    let integer_clip = i_curve::float::CurveConverter::<_, i64>::try_with_adapter(&clip, &adapter)
        .unwrap()
        .into_shape();

    let integer_result = overlay(
        integer_subject,
        integer_clip,
        OverlayRule::Intersect,
        FillRule::NonZero,
    )
    .unwrap();
    let float_result = integer_result
        .into_iter()
        .map(|shape| i_curve::float::try_convert_shape_to_float(shape, &adapter))
        .collect::<Result<Vec<_>, i_curve::float::CurveToFloatError>>()
        .unwrap();

    assert_eq!(float_result.len(), 1);
    assert_eq!(float_result[0].contours().len(), 1);
}

#[test]
fn float_overlay_supports_explicit_i64_solver() {
    let subject = float_rectangle(0.0, 0.0, 10.0, 10.0);
    let clip = float_rectangle(5.0, 2.0, 12.0, 8.0);

    let subject_only = FloatCurveOverlay::<_, i64>::try_from_subject_with_scale(&subject, 1_000.0).unwrap();
    assert_eq!(subject_only.scale(), 1_000.0);
    assert!(subject_only.conversion_report().clip.is_none());
    assert_eq!(subject_only.resolve_subject(FillRule::NonZero).len(), 1);

    let options = FloatCurveOverlayOptions::default().with_min_chord_length(0.001);
    let overlay = FloatCurveOverlay::<_, i64>::new(&subject, &clip)
        .try_with_options(options)
        .unwrap()
        .with_solver(Solver::with_precision(Precision::MEDIUM));

    assert!(overlay.scale().is_finite());
    assert!(overlay.scale() > 0.0);
    assert_eq!(overlay.options(), options);
    assert_eq!(overlay.solver().precision, Precision::MEDIUM);

    let result = overlay.overlay(OverlayRule::Intersect, FillRule::NonZero);

    assert_eq!(result.len(), 1);
}

#[test]
fn float_results_support_debug_and_container_conversions() {
    let shape = float_rectangle(0.0, 0.0, 10.0, 10.0);

    let debug = format!("{shape:?}");
    assert!(debug.contains("CurveShape"));
    assert_eq!(shape.len(), shape.contours().len());
    let _: &[i_curve::FloatCurvePath<[f64; 2]>] = shape.as_ref();

    let borrowed_segment_count = (&shape).into_iter().flat_map(IntoIterator::into_iter).count();
    assert_eq!(borrowed_segment_count, shape.segment_count());

    let taken_contours = shape.clone().into_contours();
    assert_eq!(taken_contours.len(), shape.len());

    let mut contours: Vec<_> = shape.into_iter().collect();
    let contour = contours.remove(0);
    assert_eq!(contour.len(), borrowed_segment_count);
    let _: &[i_curve::FloatCurveSegment<[f64; 2]>] = contour.as_ref();

    let owned_segments: Vec<_> = contour.clone().into_iter().collect();
    assert_eq!(owned_segments.len(), borrowed_segment_count);

    let expected_start = contour.start();
    let (start, segments) = contour.into_parts();
    assert_eq!(start, expected_start);
    assert_eq!(segments.len(), borrowed_segment_count);

    let rebuilt_contour = i_curve::FloatCurvePath::try_new(start, segments).unwrap();
    let rebuilt_from_parts: i_curve::FloatCurvePath<[f64; 2]> =
        rebuilt_contour.clone().into_parts().try_into().unwrap();
    assert_eq!(rebuilt_from_parts, rebuilt_contour);

    let shape_from_path: i_curve::FloatCurveShape<[f64; 2]> = rebuilt_contour.clone().into();
    let rebuilt_shape = i_curve::FloatCurveShape::try_new(vec![rebuilt_contour.clone()]).unwrap();
    let rebuilt_from_contours: i_curve::FloatCurveShape<[f64; 2]> = vec![rebuilt_contour].try_into().unwrap();
    assert_eq!(shape_from_path, rebuilt_shape);
    assert_eq!(rebuilt_from_contours, rebuilt_shape);

    assert_eq!(
        i_curve::FloatCurvePath::<[f64; 2]>::try_new([0.0, 0.0], vec![]),
        Err(i_curve::CurveBuildError::EmptyPath)
    );
    assert_eq!(
        i_curve::FloatCurveShape::<[f64; 2]>::try_new(vec![]),
        Err(i_curve::CurveBuildError::NoContours)
    );
}
