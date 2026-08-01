# iCurve

`iCurve` performs boolean operations on closed paths made from lines,
quadratic and cubic Bézier curves, and rational elliptic arcs. It uses robust
integer topology from [`iOverlay`](https://github.com/iShape-Rust/iOverlay)
and recomposes the result as curves instead of returning flattened polygons.

The crate supports `i16`, `i32`, and `i64` integer coordinates, float input
through a shared fixed-point adapter, and `no_std` environments with `alloc`.

## Quick start

The common integer case needs only top-level imports. A `CurveShape` contains
one or more closed contours; output shapes contain an outer contour followed
by any holes.

```rust
use i_curve::{
    overlay, FillRule, IntCurvePath, IntCurveSegment, IntCurveShape, IntPoint,
    OverlayRule,
};

fn rectangle(x0: i32, y0: i32, x1: i32, y1: i32) -> IntCurveShape<i32> {
    let start = IntPoint::new(x0, y0);
    IntCurveShape::from_path(IntCurvePath::new(
        start,
        vec![
            IntCurveSegment::Line { to: IntPoint::new(x1, y0) },
            IntCurveSegment::Line { to: IntPoint::new(x1, y1) },
            IntCurveSegment::Line { to: IntPoint::new(x0, y1) },
            IntCurveSegment::Line { to: start },
        ],
    ))
}

let result = overlay(
    rectangle(0, 0, 100, 100),
    rectangle(50, 20, 140, 80),
    OverlayRule::Intersect,
    FillRule::NonZero,
)?;
assert_eq!(result.len(), 1);
# Ok::<(), i_curve::CurveInputError>(())
```

For multiple inputs or custom precision, use `IntCurveOverlay`:

```rust
use i_curve::{
    CurveOverlayOptions, FillRule, IntCurveOverlay, IntCurvePath,
    IntCurveSegment, IntCurveShape, IntPoint, OverlayRule, Precision, Solver,
};

let start = IntPoint::new(0_i32, 0);
let shape = IntCurveShape::from_path(IntCurvePath::new(
    start,
    vec![
        IntCurveSegment::Quad {
            ctrl: IntPoint::new(50, -40),
            to: IntPoint::new(100, 0),
        },
        IntCurveSegment::Line { to: start },
    ],
));

let mut curves = IntCurveOverlay::with_capacity(shape.segment_count())
    .with_solver(Solver::with_precision(Precision::MEDIUM))
    .with_options(CurveOverlayOptions::default());
curves.add_subject(shape)?;
let result = curves.overlay(OverlayRule::Subject, FillRule::NonZero);
assert!(!result.is_empty());
# Ok::<(), i_curve::CurveInputError>(())
```

`add_subject`, `add_clip`, and `add_shape` consume their input and validate
that every contour is non-empty, connected, and closed. `overlay` consumes the
overlay builder because its prepared edge storage is a one-operation value.

## Float input

`CurveBuilder` validates float paths without quantizing them. `CurveConverter`
then chooses one safe power-of-two integer scale for the complete source shape,
or validates an explicit scale. All contours taking part in one operation must
be converted together so that they share an adapter and grid.

```rust
use i_curve::{
    CurveBuilder, CurveConverter, FillRule, IntCurveOverlay, IntCurveShape,
    OverlayRule,
};

let source = CurveBuilder::new()
    // Subject contour.
    .move_to([0.0_f64, 0.0])?
    .cubic_to([20.0, -20.0], [80.0, -20.0], [100.0, 0.0])?
    .line_to([100.0, 80.0])?
    .line_to([0.0, 80.0])?
    .close_contour()?
    // Clip contour.
    .move_to([40.0, -10.0])?
    .line_to([120.0, -10.0])?
    .line_to([120.0, 90.0])?
    .line_to([40.0, 90.0])?
    .close_contour()?
    .build()?;

let converter = CurveConverter::<_, i32>::new(source);
let (adapter, converted) = converter.into_parts();
let mut contours = converted.contours.into_iter();
let subject = IntCurveShape::from_path(contours.next().unwrap());
let clip = IntCurveShape::from_path(contours.next().unwrap());

let mut curves = IntCurveOverlay::with_capacity(8);
curves.add_subject(subject)?;
curves.add_clip(clip)?;
let result = curves.overlay(OverlayRule::Difference, FillRule::NonZero);

let first: [f64; 2] = adapter.int_to_float(&result[0].contours[0].start);
assert!(first[0].is_finite());
# Ok::<(), Box<dyn std::error::Error>>(())
```

For a fixed grid, use
`CurveConverter::<_, i32>::try_with_scale(source, scale)`. A larger scale keeps
smaller details but reduces the available source-coordinate range. Keep the
returned `FloatPointAdapter` for mapping result points back to float space.

## Elliptic and rational arcs

`float::arc::EllipticArc` is a convenient center/radii/rotation description.
`CurveBuilder::arc_to` converts it into connected, XY-monotone rational
quadratic pieces. Stored shapes use `float::arc::RationalArc`; its control
points and weights are authoritative, while its ellipse and angles are
supporting metadata.

```rust
use i_curve::float::arc::{Ellipse, EllipticArc};

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

let pieces = arc.to_rational_arcs()?;
for pair in pieces.windows(2) {
    assert_eq!(pair[0].end_point(), pair[1].start_point());
}
assert!(pieces[0].try_to_elliptic_arc(1.0e-10).is_some());
# Ok::<(), i_curve::float::arc::EllipticArcError>(())
```

Boolean snapping can move a rational endpoint away from its supporting
ellipse. `try_to_elliptic_arc` is therefore intentionally fallible and never
silently inserts connector lines. The advanced integer representation is
available under `i_curve::int::arc`.

## Precision and topology contract

Topology is determined in three fixed stages:

1. Curves receive a bounded adaptive chord approximation.
2. Exactly one curve-aware planarization pass splits nearby or intersecting
   curve pieces while preserving source parameter ranges.
3. `iOverlay` resolves integer polygon topology and may split or snap the
   chords again; provenance is split and reversed with each edge before curve
   recomposition.

`CurveOverlayOptions` controls only stage 1:

- `min_chord_length_power`: chords shorter than `2^power` integer grid units
  stop subdividing. Increasing it produces fewer, longer chords.
- `angle_tolerance_power`: a curve is locally accepted when its endpoint
  handles are collinear with the chord to a sine tolerance of roughly
  `2^-power`. Increasing it tightens the angular test and can add chords.
- `max_approximation_depth`: a hard subdivision cap. Reaching it accepts the
  current chord even if the angle test has not passed.

The `Solver` precision controls snapping in the subsequent polygon overlay.
It is independent of the curve approximation settings. Defaults are stable
for 0.1.x, but callers whose topology depends on sub-grid gaps should set both
the conversion scale and precision options explicitly.

The practical resolution is the coarsest of the float conversion grid, chord
approximation, planarization radius, and `iOverlay` snapping. Features below
that resolution can collapse. Nearly coincident curves can be merged, split,
or classified as touching after quantization or snapping; iCurve does not
promise analytic topology for separations smaller than the selected integer
resolution. Translate or rescale the input, and tighten the options, when such
features must remain distinct.

### Safe integer coordinate range

The curve kernel reserves six coordinate bits for polynomial coefficient
growth. Every endpoint and control point supplied to the integer API must stay
inside these inclusive ranges:

| Integer | Safe coordinate range |
| --- | ---: |
| `i16` | `[-1_024, 1_024]` (`[-2^10, 2^10]`) |
| `i32` | `[-67_108_864, 67_108_864]` (`[-2^26, 2^26]`) |
| `i64` | `[-288_230_376_151_711_744, 288_230_376_151_711_744]` (`[-2^58, 2^58]`) |

The storage types can represent larger coordinates, but intermediate curve
coefficients are not guaranteed safe there. This numeric range is a caller
precondition and is not checked by `add_shape`. Automatic float conversion
centers the complete bounds and selects a safe scale using the same limit.

## API layers

- Top-level re-exports cover the ordinary overlay, rules, point type, float
  builder/converter, and aliased float/integer path types.
- `i_curve::int` and `i_curve::float` contain the extended data model;
  rational-arc details live in their `arc` submodules.
- Approximation, planarization, provenance, normalization, intersection, and
  collection modules are internal and carry no compatibility promise.

No kurbo, SVG, lyon, GUI, or serialization integration is part of the 0.1.0
core. Small optional adapters can be evaluated after the path model has seen
real downstream use; SVG parsing in particular belongs in a separate adapter
crate rather than in the geometry kernel.

## Known limitations in 0.1.0

- Inputs must be closed paths; open-path clipping and stroking are out of scope.
- Topology follows the documented discrete approximation, not exact symbolic
  curve intersection.
- Float boolean results remain in the shared integer coordinate system;
  callers map result points through the returned adapter.
- Rational arcs may no longer match their supporting ellipse after snapping.

## License

`iCurve` is distributed under the MIT License. See the
[license file](https://github.com/iShape-Rust/iCurve/blob/main/LICENSE).
