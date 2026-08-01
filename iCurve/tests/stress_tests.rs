use i_curve::int::{CurvePath, CurveSegment, CurveShape};
use i_curve::{FillRule, IntCurveOverlay, IntPoint, OverlayRule, Precision, ShapeType, Solver};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};

const DEFAULT_CASES: u64 = 10_000;
const DEFAULT_SEED: u64 = 0x7a6d_4c2b_91e8_f503;
const DEFAULT_EQUIVALENCE_SEGMENT_LIMIT: usize = 256;

/// Exercises the complete curve overlay pipeline with reproducible random input.
///
/// Run with:
/// `ICURVE_STRESS_CASES=10000 ICURVE_STRESS_SEED=123 cargo test --test stress_tests -- --ignored --nocapture`
///
/// Set `ICURVE_STRESS_EQUIVALENCE_LIMIT` higher to run secondary XOR
/// equivalence checks on large overlay results.
/// Use `ICURVE_STRESS_START_CASE` to resume from a specific case index.
/// Set `ICURVE_STRESS_TRACE=1` to print every operation within a case.
/// Set `ICURVE_STRESS_PRECISION` to `absolute`, `high`, `medium-high`,
/// `medium`, `medium-low`, or `low` to override the default solver precision.
#[test]
#[ignore = "long-running randomized stress test"]
fn randomized_boolean_invariants() {
    let cases = env_u64("ICURVE_STRESS_CASES", DEFAULT_CASES);
    let base_seed = env_u64("ICURVE_STRESS_SEED", DEFAULT_SEED);
    let start_case = env_u64("ICURVE_STRESS_START_CASE", 0);
    let trace = env_u64("ICURVE_STRESS_TRACE", 0) != 0;
    let equivalence_limit = env_usize(
        "ICURVE_STRESS_EQUIVALENCE_LIMIT",
        DEFAULT_EQUIVALENCE_SEGMENT_LIMIT,
    );

    eprintln!(
        "iCurve stress test: cases={cases}, start_case={start_case}, seed={base_seed}, equivalence_limit={equivalence_limit}, precision={}",
        std::env::var("ICURVE_STRESS_PRECISION").unwrap_or_else(|_| "default".into())
    );

    for case_index in start_case..start_case.saturating_add(cases) {
        let case_seed = splitmix64(base_seed.wrapping_add(case_index));
        let mut rng = StdRng::seed_from_u64(case_seed);
        let subject = random_shape_group(&mut rng);
        let clip = random_shape_group(&mut rng);

        let outcome = catch_unwind(AssertUnwindSafe(|| {
            run_case(case_index, &mut rng, &subject, &clip, equivalence_limit, trace);
        }));

        if let Err(payload) = outcome {
            panic!(
                "stress case failed: case={case_index}, case_seed={case_seed}, base_seed={base_seed}\n\
                 reason={}\nsubject={subject:#?}\nclip={clip:#?}",
                panic_message(payload.as_ref())
            );
        }

        let completed = case_index - start_case + 1;
        if completed.is_multiple_of(100) || completed == cases {
            eprintln!("completed {completed}/{cases} cases");
        }
    }
}

fn run_case(
    case_index: u64,
    rng: &mut StdRng,
    subject: &[CurveShape<i32>],
    clip: &[CurveShape<i32>],
    equivalence_limit: usize,
    trace: bool,
) {
    let rules = [
        OverlayRule::Subject,
        OverlayRule::Clip,
        OverlayRule::Intersect,
        OverlayRule::Union,
        OverlayRule::Difference,
        OverlayRule::InverseDifference,
        OverlayRule::Xor,
    ];

    for rule in rules {
        if trace {
            eprintln!("case {case_index}: starting rule {rule}");
        }
        let result = overlay(subject, clip, rule);
        if trace {
            eprintln!("case {case_index}: completed rule {rule}");
        }
        assert_valid_result(&result, &format!("case {case_index}, rule {rule}"));
    }

    if trace {
        eprintln!("case {case_index}: starting self difference");
    }
    assert!(
        overlay(subject, subject, OverlayRule::Difference).is_empty(),
        "A - A must be empty"
    );
    if trace {
        eprintln!("case {case_index}: completed self difference; starting self xor");
    }
    assert!(
        overlay(subject, subject, OverlayRule::Xor).is_empty(),
        "A xor A must be empty"
    );
    if trace {
        eprintln!("case {case_index}: completed self xor");
    }

    let commutative_rule = match rng.random_range(0..3) {
        0 => OverlayRule::Union,
        1 => OverlayRule::Intersect,
        _ => OverlayRule::Xor,
    };
    if trace {
        eprintln!("case {case_index}: starting commutative {commutative_rule}");
    }
    let ab = overlay(subject, clip, commutative_rule);
    let ba = overlay(clip, subject, commutative_rule);
    assert_equivalent_if_bounded(
        &ab,
        &ba,
        &format!("{commutative_rule} must be commutative"),
        equivalence_limit,
    );
    if trace {
        eprintln!("case {case_index}: completed commutative {commutative_rule}");
    }

    if trace {
        eprintln!("case {case_index}: starting difference/inverse equivalence");
    }
    let difference = overlay(subject, clip, OverlayRule::Difference);
    let inverse = overlay(clip, subject, OverlayRule::InverseDifference);
    assert_equivalent_if_bounded(
        &difference,
        &inverse,
        "A - B must equal inverse(B, A)",
        equivalence_limit,
    );
    if trace {
        eprintln!("case {case_index}: completed difference/inverse equivalence");
    }
}

fn overlay(subject: &[CurveShape<i32>], clip: &[CurveShape<i32>], rule: OverlayRule) -> Vec<CurveShape<i32>> {
    let capacity = segment_count(subject) + segment_count(clip);
    let mut overlay = IntCurveOverlay::with_capacity(capacity).with_solver(stress_solver());

    for shape in subject {
        overlay.add_shape(shape.clone(), ShapeType::Subject).unwrap();
    }
    for shape in clip {
        overlay.add_shape(shape.clone(), ShapeType::Clip).unwrap();
    }

    overlay.overlay(rule, FillRule::NonZero)
}

fn stress_solver() -> Solver {
    let Ok(value) = std::env::var("ICURVE_STRESS_PRECISION") else {
        return Solver::default();
    };
    let precision = match value.as_str() {
        "absolute" => Precision::ABSOLUTE,
        "high" => Precision::HIGH,
        "medium-high" => Precision::MEDIUM_HIGH,
        "medium" => Precision::MEDIUM,
        "medium-low" => Precision::MEDIUM_LOW,
        "low" => Precision::LOW,
        _ => panic!("unsupported ICURVE_STRESS_PRECISION={value}"),
    };
    Solver::with_precision(precision)
}

fn assert_equivalent(lhs: &[CurveShape<i32>], rhs: &[CurveShape<i32>], context: &str) {
    let difference = overlay(lhs, rhs, OverlayRule::Xor);
    assert!(
        difference.is_empty(),
        "{context}; symmetric difference was {difference:#?}"
    );
}

fn assert_equivalent_if_bounded(
    lhs: &[CurveShape<i32>],
    rhs: &[CurveShape<i32>],
    context: &str,
    segment_limit: usize,
) {
    let segments = segment_count(lhs).saturating_add(segment_count(rhs));
    if segments <= segment_limit {
        assert_equivalent(lhs, rhs, context);
    }
}

fn assert_valid_result(result: &[CurveShape<i32>], context: &str) {
    for (shape_index, shape) in result.iter().enumerate() {
        assert!(
            !shape.contours.is_empty(),
            "{context}: shape {shape_index} has no contours"
        );

        for (contour_index, contour) in shape.contours.iter().enumerate() {
            assert!(
                !contour.segments.is_empty(),
                "{context}: contour {shape_index}:{contour_index} has no segments"
            );

            let mut current = contour.start;
            for (segment_index, segment) in contour.segments.iter().enumerate() {
                current = match segment {
                    CurveSegment::Line { to }
                    | CurveSegment::Quad { to, .. }
                    | CurveSegment::Cubic { to, .. } => *to,
                    CurveSegment::Arc { arc } => {
                        assert_eq!(
                            arc.control_points[0], current,
                            "{context}: disconnected arc at {shape_index}:{contour_index}:{segment_index}"
                        );
                        arc.control_points[2]
                    }
                };
            }

            assert_eq!(
                current, contour.start,
                "{context}: contour {shape_index}:{contour_index} is not closed"
            );
        }
    }
}

fn random_shape_group(rng: &mut StdRng) -> Vec<CurveShape<i32>> {
    let shape_count = rng.random_range(1..=2);
    (0..shape_count)
        .map(|_| {
            let contour_count = rng.random_range(1..=3);
            CurveShape {
                contours: (0..contour_count).map(|_| random_contour(rng)).collect(),
            }
        })
        .collect()
}

fn random_contour(rng: &mut StdRng) -> CurvePath<i32> {
    let point_count = rng.random_range(2..=8);
    let start = random_point(rng);
    let mut current = start;
    let mut segments = Vec::with_capacity(point_count);

    for index in 0..point_count {
        let to = if index + 1 == point_count {
            start
        } else {
            random_point_near(rng, current)
        };
        let segment = match rng.random_range(0..100) {
            0..=24 => CurveSegment::Line { to },
            25..=59 => CurveSegment::Quad {
                ctrl: random_control(rng, current, to),
                to,
            },
            _ => CurveSegment::Cubic {
                ctrl0: random_control(rng, current, to),
                ctrl1: random_control(rng, current, to),
                to,
            },
        };
        segments.push(segment);
        current = to;
    }

    CurvePath { start, segments }
}

fn random_point(rng: &mut StdRng) -> IntPoint<i32> {
    match rng.random_range(0..4) {
        0 => IntPoint::new(rng.random_range(-8..=8), rng.random_range(-8..=8)),
        1 => IntPoint::new(rng.random_range(-256..=256), rng.random_range(-256..=256)),
        2 => IntPoint::new(rng.random_range(-4096..=4096), rng.random_range(-4096..=4096)),
        _ => IntPoint::new(
            rng.random_range(-1_000_000..=1_000_000),
            rng.random_range(-1_000_000..=1_000_000),
        ),
    }
}

fn random_point_near(rng: &mut StdRng, current: IntPoint<i32>) -> IntPoint<i32> {
    if rng.random_range(0..5) == 0 {
        let dx = rng.random_range(-4..=4);
        let dy = rng.random_range(-4..=4);
        IntPoint::new(current.x + dx, current.y + dy)
    } else {
        random_point(rng)
    }
}

fn random_control(rng: &mut StdRng, from: IntPoint<i32>, to: IntPoint<i32>) -> IntPoint<i32> {
    match rng.random_range(0..4) {
        0 => from,
        1 => to,
        2 => IntPoint::new(
            midpoint(from.x, to.x) + rng.random_range(-8..=8),
            midpoint(from.y, to.y) + rng.random_range(-8..=8),
        ),
        _ => random_point(rng),
    }
}

fn midpoint(a: i32, b: i32) -> i32 {
    ((a as i64 + b as i64) / 2) as i32
}

fn segment_count(shapes: &[CurveShape<i32>]) -> usize {
    shapes
        .iter()
        .flat_map(|shape| &shape.contours)
        .map(|contour| contour.segments.len())
        .sum()
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    env_u64(name, default as u64).min(usize::MAX as u64) as usize
}

fn panic_message(payload: &(dyn Any + Send)) -> &str {
    if let Some(message) = payload.downcast_ref::<String>() {
        message
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        message
    } else {
        "non-string panic payload"
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
