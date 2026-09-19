use core::error::Error;
use i_curve::CurveConversionError;
use i_curve::float::{CurveConverter, CurvePath, CurveSegment, FloatCurveOverlay, FloatRectError};
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterScaleError};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

#[test]
fn rectangle_errors_preserve_the_public_cause() {
    for cause in [
        FloatRectError::CoordinatesOutOfRange,
        FloatRectError::InvalidBounds,
    ] {
        let error = CurveConversionError::from(FloatPointAdapterScaleError::InvalidRect(cause));
        assert_eq!(error, CurveConversionError::InvalidRect(cause));
        assert_eq!(CurveConversionError::from(cause), error);
        assert_eq!(error.to_string(), format!("invalid conversion bounds: {cause}"));
        let source = error.source().unwrap();
        assert_eq!(source.downcast_ref::<FloatRectError>(), Some(&cause));
        assert!(source.source().is_none());
    }
}

#[test]
fn scale_errors_preserve_their_kind_without_a_source() {
    for (adapter_error, expected) in [
        (
            FloatPointAdapterScaleError::ScaleTooLarge,
            CurveConversionError::ScaleTooLarge,
        ),
        (
            FloatPointAdapterScaleError::ScaleTooSmall,
            CurveConversionError::ScaleTooSmall,
        ),
        (
            FloatPointAdapterScaleError::ScaleNonPositive,
            CurveConversionError::ScaleNonPositive,
        ),
        (
            FloatPointAdapterScaleError::ScaleNotFinite,
            CurveConversionError::ScaleNotFinite,
        ),
    ] {
        let error = CurveConversionError::from(adapter_error);
        assert_eq!(error, expected);
        assert!(error.source().is_none());
    }
    assert!(CurveConversionError::ResourceOutsideAdapter.source().is_none());
    assert_eq!(
        CurveConversionError::ScaleTooSmall.to_string(),
        "conversion scale has a non-finite reciprocal in the input scalar type"
    );
}

fn check_scale<F: FloatNumber + core::fmt::Debug>(
    source: &[CurvePath<[F; 2]>],
    scale: F,
    expected: Result<F, CurveConversionError>,
) {
    assert_eq!(
        CurveConverter::<_, i32>::try_with_scale(source, scale).map(|converter| converter.scale()),
        expected
    );
    assert_eq!(
        FloatCurveOverlay::<_, i32>::try_from_subject_with_scale(source, scale)
            .map(|overlay| overlay.scale()),
        expected
    );
    assert_eq!(
        FloatCurveOverlay::<_, i32>::try_with_scale(source, source, scale).map(|overlay| overlay.scale()),
        expected
    );
}

fn check_scalar_scales<F: FloatNumber + core::fmt::Debug>(too_small: F, subnormal: F) {
    let start = [F::ZERO, F::ZERO];
    let point = CurvePath::try_new(start, vec![CurveSegment::Line { to: start }]).unwrap();
    let line = CurvePath::try_new(
        start,
        vec![
            CurveSegment::Line { to: [F::ONE, F::ONE] },
            CurveSegment::Line { to: start },
        ],
    )
    .unwrap();
    let point_source = [point];
    let line_source = [line];
    for source in [&[][..], &point_source[..], &line_source[..]] {
        check_scale(source, too_small, Err(CurveConversionError::ScaleTooSmall));
        check_scale(source, subnormal, Ok(subnormal));
        check_scale(source, F::from_float(8.0), Ok(F::from_float(8.0)));
        for scale in [F::ZERO, -F::ZERO, -F::ONE] {
            check_scale(source, scale, Err(CurveConversionError::ScaleNonPositive));
        }
        for scale in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            check_scale(
                source,
                F::from_float(scale),
                Err(CurveConversionError::ScaleNotFinite),
            );
        }
    }
    // Scale is not a coordinate: point bounds admit scales above MAX_COORDINATE.
    for source in [&[][..], &point_source[..]] {
        check_scale(source, F::MAX, Ok(F::MAX));
    }
}

#[test]
fn constructors_validate_reciprocals_in_f32() {
    check_scalar_scales(f32::from_bits(1), f32::MIN_POSITIVE / 2.0);
    check_scale::<f32>(&[], 1.0e-40, Err(CurveConversionError::ScaleTooSmall));
}

#[test]
fn constructors_validate_reciprocals_in_f64() {
    check_scalar_scales(f64::from_bits(1), f64::MIN_POSITIVE / 2.0);
    check_scale::<f64>(&[], 1.0e-40, Ok(1.0e-40));
}

#[test]
fn invalid_adapter_bounds_take_priority_over_invalid_scale() {
    for (bounds, cause) in [
        (
            FloatRect {
                min_x: 1.0,
                max_x: 0.0,
                min_y: 0.0,
                max_y: 0.0,
            },
            FloatRectError::InvalidBounds,
        ),
        (
            FloatRect {
                min_x: f64::INFINITY,
                max_x: 0.0,
                min_y: 0.0,
                max_y: 0.0,
            },
            FloatRectError::CoordinatesOutOfRange,
        ),
    ] {
        for scale in [f64::NAN, 0.0, f64::from_bits(1)] {
            let error =
                FloatPointAdapter::<[f64; 2], i32>::try_with_scale_and_coordinate_bits(bounds, scale, 26)
                    .err()
                    .unwrap();
            assert_eq!(
                CurveConversionError::from(error),
                CurveConversionError::InvalidRect(cause)
            );
        }
    }
}
