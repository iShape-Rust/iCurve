# iCurve

[![crates.io version](https://img.shields.io/crates/v/i_curve.svg)](https://crates.io/crates/i_curve)
[![docs.rs docs](https://docs.rs/i_curve/badge.svg)](https://docs.rs/i_curve)
[![license](https://img.shields.io/crates/l/i_curve.svg)](https://crates.io/crates/i_curve)

iCurve is a Rust library for Boolean operations on closed 2D paths made of
lines, quadratic and cubic Bézier curves, and elliptic arcs. It provides
floating-point APIs for `f32` and `f64`, a direct integer API, and selectable
`i16`, `i32`, and `i64` topology engines.

## Table of Contents

- [Why iCurve?](#why-icurve)
- [Features](#features)
- [Demo](#demo)
- [Getting Started](#getting-started)
  - [Quick Start](#quick-start)
- [Boolean Operations](#boolean-operations)
- [How It Works](#how-it-works)
- [Precision Model](#precision-model)
- [Current Limitations](#current-limitations)
- [License](#license)

&nbsp;
## Why iCurve?

- Work with curves throughout the whole operation instead of flattening them
  in application code.
- Use familiar path commands to construct closed shapes.
- Preserve line, quadratic, cubic, and rational arc segments in the result.
- Handle shapes with holes, multiple contours, and self-intersections.
- Get robust topology from the same overlay engine that powers
  [iOverlay](https://github.com/iShape-Rust/iOverlay).
- Use the library in `no_std` environments with `alloc`.

&nbsp;
## Features

- **Boolean operations**: union, intersection, difference, inverse difference,
  and xor.
- **Curve segments**: line, quadratic Bézier, cubic Bézier, and elliptic arc.
- **Coordinate APIs**: floating-point input with `[f32; 2]` or `[f64; 2]`, plus
  a direct fixed-point integer API.
- **Integer engines**: choose `i16`, `i32`, or `i64` to match the required
  coordinate range, precision, and performance profile.
- **Complex shapes**: multiple contours, holes, and self-intersections.
- **Validated paths**: non-finite, empty, and open geometry is rejected while
  building a shape.
- **Robust topology**: both operands are processed on one safe fixed-point grid.

&nbsp;
## Demo

- [Boolean Playground](https://ishape-rust.github.io/iShape-js/curve/boolean_playground.html)
- [Curve Types](https://ishape-rust.github.io/iShape-js/curve/type_curve.html)

&nbsp;
## Getting Started

Add iCurve to your `Cargo.toml`:

```toml
[dependencies]
i_curve = "^0.1"
```

Read the full [API documentation](https://docs.rs/i_curve).

### Quick Start

The example below builds a curved subject, intersects it with a rectangle, and
keeps the result as editable curve geometry:

```rust
use i_curve::{
    CurveBuildError, CurveBuilder, FillRule, FloatCurveShape, OverlayRule,
};

fn main() -> Result<(), CurveBuildError> {
    // Build the subject with one cubic Bézier edge and three straight edges.
    // close_contour() connects the last point back to the first when needed.
    let subject: FloatCurveShape<[f64; 2]> = CurveBuilder::new()
        .move_to([0.0, 0.0])?
        .cubic_to(
            [25.0, -30.0], // first control point
            [75.0, -30.0], // second control point
            [100.0, 0.0],  // end of the curved edge
        )?
        .line_to([100.0, 80.0])?
        .line_to([0.0, 80.0])?
        .close_contour()?
        .build()?;

    // The clip is another closed curve shape; here it is a rectangle.
    let clip: FloatCurveShape<[f64; 2]> = CurveBuilder::new()
        .move_to([40.0, -10.0])?
        .line_to([120.0, -10.0])?
        .line_to([120.0, 50.0])?
        .line_to([40.0, 50.0])?
        .close_contour()?
        .build()?;

    // Keep only the area shared by subject and clip. NonZero determines how
    // winding classifies the inside of every contour.
    let result = subject.overlay(
        &clip,
        OverlayRule::Intersect,
        FillRule::NonZero,
    );

    // Boolean results can contain several disconnected shapes. Each shape
    // contains one outer contour and may contain hole contours.
    assert!(!result.is_empty());

    Ok(())
}
```

`CurveBuilder` follows the usual path model: start a contour with `move_to`,
append segments, then call `close_contour`. Start another contour on the same
builder to add a hole or another boundary. The `overlay` call returns a
`Vec<FloatCurveShape<P>>` because one operation may produce several disconnected
shapes.

&nbsp;
## Boolean Operations

The subject is **A** and the clip is **B**:

| Rule | Result |
| --- | --- |
| `Union` | A ∪ B |
| `Intersect` | A ∩ B |
| `Difference` | A − B |
| `InverseDifference` | B − A |
| `Xor` | Area belonging to A or B, but not both |

Choose the operation with `OverlayRule` and choose how contour winding defines
the interior with `FillRule`. The Quick Start above is the complete basic API:
build two shapes and call `overlay` on the subject.

&nbsp;
## How It Works

iCurve keeps the public API in floating-point curve space, while its topology
pipeline uses a shared discrete coordinate system for predictable results.

1. **Validate the paths.** Every contour must be finite, non-empty, and closed.
   Elliptic arcs added through `CurveBuilder` are represented internally as
   connected rational quadratic arc segments.
2. **Create one coordinate grid.** The combined bounds of the subject and clip
   determine the largest safe power-of-two scale for the selected integer
   engine. The basic float `overlay` method uses `i32`; `overlay_as` can select
   `i16`, `i32`, or `i64`. Both operands are translated and rounded through the
   same adapter, so they cannot disagree about the location of a shared point.
3. **Normalize the curves.** Lines, Bézier curves, and rational arcs are split
   into canonical pieces suitable for intersection processing. Each piece keeps
   its source curve id and source parameter interval.
4. **Build the topology.** Curves receive a bounded adaptive chord
   approximation, followed by curve-aware planarization around intersections.
   The generated edges still carry provenance back to the original curve
   fragments.
5. **Resolve the Boolean rule.** [iOverlay](https://github.com/iShape-Rust/iOverlay)
   classifies the planar edge graph with the selected `OverlayRule` and
   `FillRule`, then traces the boundaries that belong in the result.
6. **Reconstruct curves.** Adjacent output spans from the same source are joined
   into maximal runs and restored as lines, quadratic or cubic Bézier curves,
   or rational arcs. Finally, points are mapped back to the input float type.

This design uses polygonal chords to determine topology without turning the
public result into a polygon. The returned boundaries remain curve segments and
can be rendered, edited, or used as input to another Boolean operation.

&nbsp;
## Precision Model

The shared fixed-point grid makes topology robust and reproducible for one
operation, but it is still a discrete model. Features smaller than the effective
grid resolution can collapse or be classified as touching, and snapping can
slightly adjust reconstructed curve geometry.

The convenience `overlay` method selects the scale automatically. Applications
that depend on very small gaps or need several operations to use exactly the
same quantization can configure an explicit scale through `FloatCurveOverlay`.
See the [API documentation](https://docs.rs/i_curve/latest/i_curve/struct.FloatCurveOverlay.html)
for advanced precision, solver, and conversion diagnostics.

The float API always returns the original point type, regardless of the integer
engine used internally. Applications that already store fixed-point geometry
can skip float conversion and use `CurveShape<I>` with `IntCurveOverlay<I>`
directly, where `I` is `i16`, `i32`, or `i64`.

&nbsp;
## Current Limitations

- Inputs must be closed paths; open-path clipping and stroking are not
  supported.
- Topology uses the discrete precision model described above, not exact symbolic
  curve intersection.
- Rational arcs may no longer lie exactly on their supporting ellipse after
  snapping.
- SVG parsing, rendering, GUI integration, and serialization are outside the
  core crate.

&nbsp;
## License

iCurve is distributed under the MIT License. See
[`LICENSE`](https://github.com/iShape-Rust/iCurve/blob/main/LICENSE).
