# iCurve

`iCurve` performs Boolean operations directly on closed 2D curve paths. The
primary API accepts `f32` or `f64` coordinates and returns curves in the same
coordinate type—there is no integer conversion to manage in application code.

Supported segments:

- lines;
- quadratic Bézier curves;
- cubic Bézier curves;
- elliptic arcs represented as rational quadratic curves.

The available operations are union, intersection, difference, inverse
difference, and xor. Shapes may contain multiple contours, including holes and
self-intersections.

Internally, iCurve maps both operands to one safe fixed-point grid and uses
[`iOverlay`](https://github.com/iShape-Rust/iOverlay) for robust topology. The
result is then reconstructed and returned through the float curve API.

## Installation

```toml
[dependencies]
i_curve = "0.1"
```

## Float quick start

Build each closed shape with `CurveBuilder`, then call `overlay` on the subject.
The example below intersects a cubic shape with a rectangle:

```rust
use i_curve::{CurveBuilder, FillRule, FloatCurveShape, OverlayRule};

fn subject() -> Result<FloatCurveShape<[f64; 2]>, i_curve::CurveBuildError> {
    CurveBuilder::new()
        .move_to([0.0, 0.0])?
        .cubic_to([25.0, -30.0], [75.0, -30.0], [100.0, 0.0])?
        .line_to([100.0, 80.0])?
        .line_to([0.0, 80.0])?
        .close_contour()?
        .build()
}

fn rectangle(
    min: [f64; 2],
    max: [f64; 2],
) -> Result<FloatCurveShape<[f64; 2]>, i_curve::CurveBuildError> {
    CurveBuilder::new()
        .move_to([min[0], min[1]])?
        .line_to([max[0], min[1]])?
        .line_to(max)?
        .line_to([min[0], max[1]])?
        .close_contour()?
        .build()
}

let subject = subject()?;
let clip = rectangle([40.0, -10.0], [120.0, 50.0])?;

let result = subject.overlay(
    &clip,
    OverlayRule::Intersect,
    FillRule::NonZero,
);

assert!(!result.is_empty());
# Ok::<(), i_curve::CurveBuildError>(())
```

Every returned `FloatCurvePath` is a validated, non-empty closed contour.

`[f32; 2]` works the same way—use an `f32` literal when constructing the first
point so Rust infers the desired scalar type:

```rust
use i_curve::CurveBuilder;

let shape = CurveBuilder::new()
    .move_to([0.0_f32, 0.0])?
    .line_to([10.0, 0.0])?
    .line_to([10.0, 10.0])?
    .close_contour()?
    .build()?;

let _: i_curve::FloatCurveShape<[f32; 2]> = shape;
# Ok::<(), i_curve::CurveBuildError>(())
```

## Building float shapes

`CurveBuilder` uses path-style commands:

| Method | Segment |
| --- | --- |
| `move_to(point)` | starts a contour |
| `line_to(to)` | line |
| `quad_to(control, to)` | quadratic Bézier |
| `cubic_to(control0, control1, to)` | cubic Bézier |
| `arc_to(arc)` | elliptic arc |
| `rational_arc_to(arc)` | an existing rational arc |
| `close_contour()` | closes and finishes the current contour |
| `build()` | validates and returns the shape |

Every contour must contain at least one segment and must be closed.
`close_contour()` adds a final line only when the current endpoint differs from
the start point. To build a shape with holes or disjoint contours, continue
with another `move_to` after closing the current contour.

All input points and arc values must be finite. Invalid or open paths are
reported as `CurveBuildError` during construction.

Commands mutate the builder and return `&mut Self`, so the same API also works
without reassignment in dynamic loops. A successful `build()` takes the
accumulated contours and resets the builder for reuse:

```rust
use i_curve::CurveBuilder;

let points = [[0.0_f64, 0.0], [10.0, 0.0], [10.0, 10.0]];
let mut builder = CurveBuilder::new();

builder.move_to(points[0])?;
for point in &points[1..] {
    builder.line_to(*point)?;
}

let shape = builder.close_contour()?.build()?;
assert_eq!(shape.contours().len(), 1);
# Ok::<(), i_curve::CurveBuildError>(())
```

## Boolean operations

For the common shape case, `overlay` is an inherent method, so no extension
trait import is needed:

```rust
use i_curve::{FillRule, FloatCurveShape, OverlayRule};

# fn example(subject: &FloatCurveShape<[f64; 2]>, clip: &FloatCurveShape<[f64; 2]>) {
let result = subject.overlay(
    clip,
    OverlayRule::Intersect,
    FillRule::NonZero,
);
# let _ = result;
# }
```

`OverlayRule` provides:

- `Union` — area in either shape;
- `Intersect` — area shared by both shapes;
- `Difference` — subject minus clip;
- `InverseDifference` — clip minus subject;
- `Xor` — area belonging to exactly one shape;
- `Subject` and `Clip` — resolve one operand using the selected fill rule.

The return type is `Vec<FloatCurveShape<P>>`, where `P` is the same point type
used by the input. A result can contain multiple shapes, each containing one or
more closed contours.

`FloatCurvePath` also provides `overlay` directly. For a slice, array, `Vec`,
or another `CurveResource`, import `CurveResourceOverlayExt`. Every path in one
resource belongs to the same subject or clip operand:

```rust
use i_curve::{CurveResourceOverlayExt, FillRule, OverlayRule};

# fn example(
#     subjects: &[i_curve::FloatCurveShape<[f64; 2]>],
#     clip: &i_curve::FloatCurveShape<[f64; 2]>,
# ) {
let result = subjects.overlay(
    clip,
    OverlayRule::Intersect,
    FillRule::NonZero,
);
# let _ = result;
# }
```

Use `FloatCurveOverlay` when you need to configure the conversion scale, curve
approximation, or topology solver:

```rust
use i_curve::{
    CurveBuilder, FillRule, FloatCurveOverlay, FloatCurveOverlayOptions,
    OverlayRule, Precision, Solver,
};

let subject = CurveBuilder::new()
    .move_to([0.0_f64, 0.0])?
    .quad_to([50.0, -40.0], [100.0, 0.0])?
    .close_contour()?
    .build()?;

let clip = CurveBuilder::new()
    .move_to([30.0_f64, -10.0])?
    .line_to([80.0, -10.0])?
    .line_to([80.0, 10.0])?
    .close_contour()?
    .build()?;

let result = FloatCurveOverlay::<_, i32>::try_with_scale(
    &subject,
    &clip,
    10_000.0,
)?
.try_with_options(
    FloatCurveOverlayOptions::default()
        .with_min_chord_length(0.001)
        .with_angle_tolerance(0.125)
        .with_max_approximation_depth(16),
)?
.with_solver(Solver::with_precision(Precision::MEDIUM))
.overlay(OverlayRule::Difference, FillRule::NonZero);

assert!(!result.is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The default convenience API automatically chooses the largest safe
power-of-two scale from the combined bounds of both inputs. An explicit scale
is useful when several independent operations must share the same grid
resolution. A larger scale preserves smaller features but reduces the safe
coordinate range; unsafe, non-positive, and non-finite scales return
`CurveConversionError`.

`FloatCurveOverlayOptions` expresses the minimum chord length in input
coordinates. Leaving it as `None` preserves the scale-relative default. The
fallible `try_with_options` rejects non-finite or out-of-range tolerances and
subdivision depths above `CurveOverlayOptions::MAX_APPROXIMATION_DEPTH` (16).
Use `scale()` to inspect the effective float-to-integer conversion scale.

The direct `FloatCurveOverlay` API makes the integer engine explicit: use
`FloatCurveOverlay::<_, i32>` for the standard engine or
`FloatCurveOverlay::<_, i64>` for the wider one.

## Reading the result

The float result is exposed through three top-level aliases:

- `FloatCurveShape<P>` contains contours;
- `FloatCurvePath<P>` has a start point and segments;
- `FloatCurveSegment<P>` is `Line`, `Quad`, `Cubic`, or `Arc`.

```rust
use i_curve::FloatCurveSegment;

# fn inspect(result: &[i_curve::FloatCurveShape<[f64; 2]>]) {
for shape in result {
    for contour in shape {
        let mut current = contour.start();

        for segment in contour {
            match segment {
                FloatCurveSegment::Line { to } => {
                    // Render a line from `current` to `to`.
                }
                FloatCurveSegment::Quad { ctrl, to } => {
                    // Render a quadratic Bézier.
                }
                FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => {
                    // Render a cubic Bézier.
                }
                FloatCurveSegment::Arc { arc } => {
                    // `control_points` and `weights` define the rational curve.
                    let _ = (arc.control_points, arc.weights);
                }
            }
            current = segment.end_point();
        }

        assert_eq!(current, contour.start());
    }
}
# }
```

Float shapes, paths, segments, and arcs implement `Debug`. Shapes and paths
also provide `len`, `is_empty`, `iter`, `AsRef`, and owned or borrowed
`IntoIterator` implementations. Use `into_contours()` or `into_parts()` to
take a result apart without cloning, then `try_new` to validate and rebuild it:

```rust
use i_curve::{FloatCurvePath, FloatCurveShape};

# fn rebuild(shape: FloatCurveShape<[f64; 2]>) -> Result<FloatCurveShape<[f64; 2]>, i_curve::CurveBuildError> {
let mut contours = Vec::new();
for contour in shape {
    let (start, segments) = contour.into_parts();
    // Transform `segments` here without cloning them.
    contours.push(FloatCurvePath::try_new(start, segments)?);
}
FloatCurveShape::try_new(contours)
# }
```

## Elliptic arcs

Use `EllipticArc` to describe an arc by center, radii, rotation, and angles.
Rotation and angles are in radians; a positive sweep is counter-clockwise and a
negative sweep is clockwise.

```rust
use i_curve::{CurveBuilder, FloatCurveShape};
use i_curve::float::arc::{Ellipse, EllipticArc};

let circle = EllipticArc {
    ellipse: Ellipse {
        center: [50.0_f64, 50.0],
        radius_x: 40.0,
        radius_y: 40.0,
        rotation: 0.0,
    },
    start_angle: 0.0,
    sweep_angle: core::f64::consts::TAU,
};

let shape: FloatCurveShape<[f64; 2]> = CurveBuilder::new()
    .move_to(circle.start_point())?
    .arc_to(circle)?
    .close_contour()?
    .build()?;

assert_eq!(shape.contours().len(), 1);
# Ok::<(), i_curve::CurveBuildError>(())
```

The builder converts an elliptic arc into connected, XY-monotone rational
quadratic pieces. In a returned `FloatCurveSegment::Arc`, the rational control
points and weights are the authoritative geometry. Boolean snapping can move
that geometry slightly away from its supporting ellipse, so
`RationalArc::try_to_elliptic_arc` is intentionally fallible.

## Precision model

iCurve uses a discrete topology pipeline rather than exact symbolic curve
intersection:

1. Float coordinates are mapped to a shared fixed-point grid.
2. Curves are approximated and planarized for topology processing.
3. `iOverlay` resolves the Boolean operation.
4. Surviving curve fragments are reconstructed in float coordinates.

Features smaller than the effective grid and snapping resolution can collapse
or be classified as touching. For geometry that depends on very small gaps,
keep coordinates near the origin and use an explicit scale and solver
precision. The same explicit scale should be reused when separate operations
need reproducible quantization.

## Advanced integer API

Applications that already store validated fixed-point geometry can use
`CurveShape`, `IntCurveOverlay`, and the extended types under `i_curve::int`.
`CurveConverter` and `FloatPointAdapter` are available under `i_curve::float`
for manual conversion workflows. Engine-generic APIs use the sealed
`i_curve::int::CurveInt` trait, implemented for `i16`, `i32`, and `i64`.
`CurveConverter::new(&resource)` accepts the same paths, shapes, and collections
as the float overlay API. It flattens their paths into one integer shape;
contours that collapse completely on the selected grid are omitted.

The integer layer is not required for ordinary `f32`/`f64` use. Its coordinate
limits and approximation options are documented in the
[`int` module](https://docs.rs/i_curve/latest/i_curve/int/index.html).

## API evolution

Public options structs and error enums are non-exhaustive so that new settings
and diagnostics can be added without a major release. Construct options from
`Default` and override values through `with_*` methods. When matching an error,
include a wildcard arm for variants introduced by future versions.

## Current limitations

- Inputs must be closed paths; open-path clipping and stroking are not
  supported.
- Topology follows the documented discrete precision model, not analytic curve
  intersection.
- Rational arcs may no longer lie exactly on their supporting ellipse after
  snapping.
- SVG parsing, rendering, GUI integration, and serialization are outside the
  core crate.

## License

`iCurve` is distributed under the MIT License. See
[`LICENSE`](https://github.com/iShape-Rust/iCurve/blob/main/LICENSE).
