# iCurve

`iCurve` performs robust boolean operations on closed paths made from lines,
quadratic and cubic Bezier curves, and rational elliptic arcs. It uses integer
geometry internally and delegates polygon topology extraction to
[`iOverlay`](https://github.com/iShape-Rust/iOverlay), while preserving the
curved segment representation in the result.

> The crate is currently experimental. The geometry kernel is usable, but the
> public API may still change before the first stable release.

## Features

- Union, intersection, difference, inverse difference, and XOR.
- Even-odd, non-zero, positive, and negative fill rules.
- Lines, quadratic Beziers, cubic Beziers, and rational elliptic arcs.
- Integer kernels for `i16`, `i32`, and `i64`.
- Float input with automatic or explicit fixed-point scaling.
- `no_std` support with `alloc`.

## Integer quick start

The integer API is the direct path to the boolean kernel. Each input shape can
contain multiple closed contours. Add shapes as either `Subject` or `Clip`,
then select an `OverlayRule` and a `FillRule`.

The current API uses point, fill-rule, and overlay-rule types from `iOverlay`,
so applications should include both crates as dependencies. The capacity
passed to `IntCurveOverlay::new` is an allocation hint and should approximately
match the total number of input segments.

```rust
use i_curve::int::{
    bool::overlay::IntCurveOverlay,
    curve::{path::CurvePath, segment::CurveSegment, shape::CurveShape},
};
use i_overlay::{
    core::{fill_rule::FillRule, overlay::ShapeType, overlay_rule::OverlayRule},
    i_shape::int::IntPoint,
};

let subject = CurveShape {
    contours: vec![CurvePath {
        start: IntPoint::new(0_i32, 0),
        segments: vec![
            CurveSegment::Line {
                to: IntPoint::new(100, 0),
            },
            CurveSegment::Quad {
                ctrl: IntPoint::new(130, 50),
                to: IntPoint::new(100, 100),
            },
            CurveSegment::Line {
                to: IntPoint::new(0, 100),
            },
            CurveSegment::Quad {
                ctrl: IntPoint::new(-30, 50),
                to: IntPoint::new(0, 0),
            },
        ],
    }],
};

let clip = CurveShape {
    contours: vec![CurvePath {
        start: IntPoint::new(50, -20),
        segments: vec![
            CurveSegment::Line {
                to: IntPoint::new(120, -20),
            },
            CurveSegment::Line {
                to: IntPoint::new(120, 120),
            },
            CurveSegment::Line {
                to: IntPoint::new(50, 120),
            },
            CurveSegment::Line {
                to: IntPoint::new(50, -20),
            },
        ],
    }],
};

let mut overlay = IntCurveOverlay::new(8);
overlay.add_shape(subject, ShapeType::Subject);
overlay.add_shape(clip, ShapeType::Clip);

let result = overlay.overlay(OverlayRule::Intersect, FillRule::NonZero);
assert!(!result.is_empty());
```

The result is a `Vec<CurveShape<I>>`. The first contour of each shape is its
outer boundary; the remaining contours are holes. A contour is closed when the
endpoint of its last segment equals its `start` point.

## Float input

`CurveBuilder` creates float paths without quantization. `CurveConverter` then
chooses a safe integer scale for the complete source shape, or uses an explicit
scale supplied by the caller.

It is important to convert all contours participating in one overlay with the
same adapter. The example below builds the subject and clip as two contours,
converts them together, and separates them afterwards.

```rust
use i_curve::{
    float::curve::{
        arc::{Ellipse, EllipticArc},
        builder::CurveBuilder,
        converter::CurveConverter,
    },
    int::{bool::overlay::IntCurveOverlay, curve::shape::CurveShape},
};
use i_overlay::core::{
    fill_rule::FillRule, overlay::ShapeType, overlay_rule::OverlayRule,
};

let ellipse = EllipticArc {
    ellipse: Ellipse {
        center: [0.0_f64, 0.0],
        radius_x: 80.0,
        radius_y: 50.0,
        rotation: 0.25,
    },
    start_angle: 0.0,
    sweep_angle: core::f64::consts::TAU,
};

let source = CurveBuilder::new()
    // Subject contour.
    .move_to(ellipse.start_point()).unwrap()
    .arc_to(ellipse).unwrap()
    .close_contour().unwrap()
    // Clip contour.
    .move_to([-20.0, -70.0]).unwrap()
    .line_to([90.0, -70.0]).unwrap()
    .line_to([90.0, 70.0]).unwrap()
    .line_to([-20.0, 70.0]).unwrap()
    .close_contour().unwrap()
    .build().unwrap();

let converter = CurveConverter::<_, i32>::new(source);
let (adapter, int_shape) = converter.into_parts();
let mut contours = int_shape.contours.into_iter();

let subject = CurveShape {
    contours: vec![contours.next().unwrap()],
};
let clip = CurveShape {
    contours: vec![contours.next().unwrap()],
};

let mut overlay = IntCurveOverlay::new(8);
overlay.add_shape(subject, ShapeType::Subject);
overlay.add_shape(clip, ShapeType::Clip);
let result = overlay.overlay(OverlayRule::Difference, FillRule::NonZero);

// Keep the adapter to map result coordinates back to the float coordinate
// system. Curved segments retain their control points and rational weights.
let first_point = adapter.int_to_float(&result[0].contours[0].start);
println!("first result point: {first_point:?}");
```

For a fixed scale, use
`CurveConverter::<_, i32>::try_with_scale(source, scale)`. A larger scale
preserves smaller details but reduces the available coordinate range.

## Elliptic and rational arcs

`EllipticArc` is a convenient input description: center, radii, rotation,
start angle, and sweep angle. Shapes themselves store only `RationalArc`.
`CurveBuilder::arc_to` converts an elliptic arc into connected, XY-monotone
rational quadratic pieces.

```rust
use i_curve::float::curve::arc::{Ellipse, EllipticArc};

let arc = EllipticArc {
    ellipse: Ellipse {
        center: [0.0_f64, 0.0],
        radius_x: 100.0,
        radius_y: 60.0,
        rotation: 0.3,
    },
    start_angle: 0.0,
    sweep_angle: core::f64::consts::PI,
};

let pieces = arc.to_rational_arcs().unwrap();
for pair in pieces.windows(2) {
    assert_eq!(pair[0].end_point(), pair[1].start_point());
}

// This succeeds only while the authoritative rational geometry still lies
// on its supporting ellipse within the requested tolerance.
let elliptic = pieces[0].try_to_elliptic_arc(1.0e-10);
assert!(elliptic.is_some());
```

For `RationalArc`, `control_points` and `weights` are the authoritative
geometry. The ellipse and angles are supporting metadata. Intersections and
snapping may move an endpoint away from the supporting ellipse, while the
rational curve remains connected to its neighbouring segments. Consequently,
`try_to_elliptic_arc` may return `None` after a boolean operation. This avoids
silently introducing small connector lines or gaps.

Use `CurveBuilder::rational_arc_to` to add an already materialized rational arc
without reconstructing its endpoints or weights from ellipse metadata.

## How it works

1. Float input is quantized into a shared integer coordinate system.
2. Curves are decomposed into simple canonical pieces.
3. Intersections split the curves until their chords preserve the same planar
   topology as the curved geometry.
4. The chords are processed by `iOverlay`, together with provenance identifying
   their source curves.
5. Consecutive output edges with common provenance are recomposed into maximal
   line, Bezier, or rational-arc runs.

The polygon topology is therefore resolved by the robust integer overlay
engine, while the output boundary retains curved segments instead of being
flattened into polylines.

## Current limitations

- The API is experimental and currently exposes the integer overlay pipeline.
- All input contours must be closed. `CurveBuilder` reports unclosed paths.
- Features smaller than the selected integer grid can collapse during
  conversion.
- Conversion from a snapped `RationalArc` back to `EllipticArc` is intentionally
  fallible.

## License

`iCurve` is distributed under the MIT License. See [LICENSE](LICENSE).
