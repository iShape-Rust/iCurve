use core::fmt::Debug;
use i_curve::float::arc::{Ellipse, RationalArc};
use i_curve::float::{CurveConverter, CurvePath, CurveSegment, CurveShape, try_convert_shape_to_float};
use i_curve::int::{CurveInt, CurveSegment as IntSegment, CurveShape as IntShape, IntPoint};
use i_curve::{FillRule, FloatCurveOverlay};
use i_overlay::i_float::float::number::FloatNumber;

fn bezier_path<F: FloatNumber + Debug>(min: F, max: F) -> CurvePath<[F; 2]> {
    let middle = min * F::HALF + max * F::HALF;
    CurvePath::try_new(
        [min, min],
        vec![
            CurveSegment::Quad {
                ctrl: [max, min],
                to: [max, max],
            },
            CurveSegment::Cubic {
                ctrl0: [middle, max],
                ctrl1: [min, middle],
                to: [min, min],
            },
        ],
    )
    .unwrap()
}

fn points<F: FloatNumber>(path: &CurvePath<[F; 2]>) -> Vec<[F; 2]> {
    let mut result = vec![path.start()];
    for segment in path.segments() {
        match segment {
            CurveSegment::Line { to } => result.push(*to),
            CurveSegment::Quad { ctrl, to } => result.extend([*ctrl, *to]),
            CurveSegment::Cubic { ctrl0, ctrl1, to } => result.extend([*ctrl0, *ctrl1, *to]),
            CurveSegment::Arc { arc } => result.extend(arc.control_points),
        }
    }
    result
}

fn assert_coordinate_budget<I: CurveInt>(shape: &IntShape<I>) {
    // iCurve reserves six bits, independently of the adapter's general budget.
    let limit = I::ONE << (I::BITS - 6);
    let check = |point: IntPoint<I>| {
        assert!(point.x >= -limit && point.x <= limit);
        assert!(point.y >= -limit && point.y <= limit);
    };
    for path in &shape.contours {
        assert!(path.is_closed());
        check(path.start);
        for segment in &path.segments {
            match segment {
                IntSegment::Line { to } => check(*to),
                IntSegment::Quad { ctrl, to } => {
                    check(*ctrl);
                    check(*to);
                }
                IntSegment::Cubic { ctrl0, ctrl1, to } => {
                    check(*ctrl0);
                    check(*ctrl1);
                    check(*to);
                }
                IntSegment::Arc { arc } => {
                    check(arc.ellipse.center);
                    for point in arc.control_points {
                        check(point);
                    }
                }
            }
        }
    }
}

fn assert_close<F: FloatNumber>(actual: F, expected: F, grid_step: F) {
    assert!(actual.is_finite());
    let actual = actual.to_f64();
    let expected = expected.to_f64();
    let epsilon = if F::BITS == 32 {
        f32::EPSILON as f64
    } else {
        f64::EPSILON
    };
    // Half a grid cell plus float error from centering and reconstructing a
    // coordinate. A fixed absolute epsilon would hide failures on tiny inputs.
    let tolerance = grid_step.to_f64() * 0.5 + 4.0 * epsilon * actual.abs().max(expected.abs());
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:e}, expected={expected:e}, tolerance={tolerance:e}"
    );
}

fn round_trip<F: FloatNumber + Debug, I: CurveInt + Debug>(
    source: &CurvePath<[F; 2]>,
    converter: CurveConverter<[F; 2], I>,
) -> CurveShape<[F; 2]> {
    assert!(converter.scale().is_finite() && converter.scale() > F::ZERO);
    assert!(converter.adapter().inv_scale().is_finite() && converter.adapter().inv_scale() > F::ZERO);
    assert!(!converter.report().has_degeneracies());
    assert_coordinate_budget(converter.shape());
    let (adapter, integer, _) = converter.into_parts();
    let restored = try_convert_shape_to_float(integer, &adapter).unwrap();
    assert_eq!(restored.contours().len(), 1);
    let path = &restored.contours()[0];
    assert_eq!(path.segments().last().unwrap().end_point(), path.start());
    assert_eq!(path.len(), source.len());
    for (actual, expected) in path.segments().iter().zip(source.segments()) {
        assert_eq!(core::mem::discriminant(actual), core::mem::discriminant(expected));
    }
    let actual = points(path);
    let expected = points(source);
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.into_iter().zip(expected) {
        for axis in 0..2 {
            assert_close(actual[axis], expected[axis], adapter.inv_scale());
        }
    }
    restored
}

fn tiny_beziers<F: FloatNumber + Debug, I: CurveInt + Debug>(radius: F) {
    let path = bezier_path(-radius, radius);
    let converter = CurveConverter::<_, I>::try_new(&path).unwrap();
    let cap = F::from_float(2.0_f64.powi(F::MAX_EXP - 1));
    assert!(converter.scale() <= cap);
    // This fixture requires a scale above the cap even for the i16 engine.
    assert_eq!(converter.scale(), cap);
    round_trip(&path, converter);
}

fn subnormal_contour<F: FloatNumber + Debug, I: CurveInt + Debug>(smallest: F) {
    let start = [F::ZERO, F::ZERO];
    let path = CurvePath::try_new(
        start,
        vec![
            CurveSegment::Line {
                to: [smallest, F::ZERO],
            },
            CurveSegment::Line {
                to: [smallest, smallest],
            },
            CurveSegment::Line {
                to: [F::ZERO, smallest],
            },
            CurveSegment::Line { to: start },
        ],
    )
    .unwrap();
    let converter = CurveConverter::<_, I>::try_new(&path).unwrap();
    assert!(converter.scale().is_finite() && converter.scale() > F::ONE);
    assert!(converter.adapter().inv_scale().is_finite() && converter.adapter().inv_scale() > F::ZERO);
    assert!(converter.shape().contours.is_empty());
    let report = converter.report();
    assert_eq!(report.contour_count, 1);
    assert_eq!(report.collapsed_contour_count, 1);
    assert_eq!(report.collapsed_segment_count, 4);
    assert_eq!(report.linearized_arc_count, 0);
    assert!(report.has_degeneracies());
    let overlay = FloatCurveOverlay::<_, I>::try_from_subject(&path).unwrap();
    assert_eq!(overlay.conversion_report().subject, converter.report());
    assert!(overlay.resolve_subject(FillRule::NonZero).is_empty());
}

fn boundary_beziers<F: FloatNumber + Debug, I: CurveInt + Debug>(below_limit: F) {
    let limit = F::MAX_COORDINATE;
    for (min, max) in [(-limit, limit), (below_limit, limit), (-limit, -below_limit)] {
        let path = bezier_path(min, max);
        round_trip(&path, CurveConverter::<_, I>::try_new(&path).unwrap());
    }
}

fn outward_rounding<F: FloatNumber + Debug, I: CurveInt + Debug>() {
    // At scale one and zero offset, +/-1.5 must snap outwards to +/-2.
    // Those results are valid even though they leave the original bounds.
    let radius = F::from_float(1.5);
    let path = bezier_path(-radius, radius);
    let restored = round_trip(
        &path,
        CurveConverter::<_, I>::try_with_scale(&path, F::ONE).unwrap(),
    );
    assert_eq!(restored.contours()[0].start(), [-F::TWO, -F::TWO]);
    assert_eq!(restored.contours()[0].segments()[0].end_point(), [F::TWO, F::TWO]);
}

fn quarter_circle<F: FloatNumber + Debug>(radius: F, rotation: F) -> CurvePath<[F; 2]> {
    let start = [radius, F::ZERO];
    let arc = RationalArc {
        ellipse: Ellipse {
            center: [F::ZERO, F::ZERO],
            radius_x: radius,
            radius_y: radius,
            rotation,
        },
        control_points: [start, [radius, radius], [F::ZERO, radius]],
        weights: [F::ONE, F::HALF.sqrt(), F::ONE],
        start_angle: -rotation,
        sweep_angle: F::from_float(core::f64::consts::FRAC_PI_2),
    };
    CurvePath::try_new(
        start,
        vec![CurveSegment::Arc { arc }, CurveSegment::Line { to: start }],
    )
    .unwrap()
}

fn arc_round_trip<F: FloatNumber + Debug, I: CurveInt + Debug>(radius: F, rotation: F) {
    // A circle's world-space quarter stays the same when its supporting frame
    // rotates; shift the local phase to retain the authoritative control points.
    let path = quarter_circle(radius, rotation);
    let converter = CurveConverter::<_, I>::try_new(&path).unwrap();
    let step = converter.adapter().inv_scale();
    let restored = round_trip(&path, converter);
    let CurveSegment::Arc { arc } = &restored.contours()[0].segments()[0] else {
        panic!("representable arc must retain its segment kind");
    };
    assert!(arc.ellipse.radius_x > F::ZERO && arc.ellipse.radius_y > F::ZERO);
    // Both axis components are rounded: their combined error is at most
    // sqrt(2) times the error of rounding one coordinate.
    assert_close(arc.ellipse.radius_x, radius, step * F::TWO.sqrt());
    assert_close(arc.ellipse.radius_y, radius, step * F::TWO.sqrt());
    assert_close(arc.ellipse.rotation, rotation, F::ZERO);
    assert!(arc.start_angle.is_finite() && arc.sweep_angle.is_finite());
    assert!(arc.sweep_angle > F::ZERO);
    assert!(arc.validate().is_ok());
}

macro_rules! cases {
    ($name:ident, $float:ty, $int:ty, $tiny:expr) => {
        mod $name {
            use super::*;

            #[test]
            fn automatic_scale_caps_without_losing_representable_beziers() {
                tiny_beziers::<$float, $int>($tiny);
            }

            #[test]
            fn subnormal_geometry_reports_collapse() {
                subnormal_contour::<$float, $int>(<$float>::from_bits(1));
            }

            #[test]
            fn coordinate_limits_and_adjacent_floats_round_trip() {
                let limit = <$float>::MAX_COORDINATE;
                boundary_beziers::<$float, $int>(<$float>::from_bits(limit.to_bits() - 1));
            }

            #[test]
            fn rounding_can_leave_original_bounds() {
                outward_rounding::<$float, $int>();
            }

            #[test]
            fn ordinary_arc_round_trips() {
                arc_round_trip::<$float, $int>(1.0, 0.0);
            }

            #[test]
            fn large_arc_round_trips() {
                arc_round_trip::<$float, $int>(<$float>::MAX_COORDINATE / 4.0, 0.0);
            }

            #[test]
            fn tiny_representable_arc_round_trips() {
                arc_round_trip::<$float, $int>($tiny, 0.0);
            }

            #[test]
            fn rotated_arc_preserves_radius_and_direction_in_each_quadrant() {
                for radius in [1.0, $tiny, <$float>::MAX_COORDINATE / 4.0] {
                    for quarter_turns in [-3.0, -1.0, 1.0, 3.0] {
                        let rotation = (quarter_turns * core::f64::consts::FRAC_PI_4) as $float;
                        arc_round_trip::<$float, $int>(radius, rotation);
                    }
                }
            }
        }
    };
}

cases!(f32_i16, f32, i16, 1.0e-36);
cases!(f32_i32, f32, i32, 1.0e-36);
cases!(f32_i64, f32, i64, 1.0e-36);
cases!(f64_i16, f64, i16, 1.0e-307);
cases!(f64_i32, f64, i32, 1.0e-307);
cases!(f64_i64, f64, i64, 1.0e-307);
