use crate::float::curve::arc::{Ellipse, RationalArc};
use crate::float::curve::path::CurvePath as FloatCurvePath;
use crate::float::curve::segment::CurveSegment as FloatCurveSegment;
use crate::float::curve::shape::CurveShape as FloatCurveShape;
use crate::float::resource::{CurveResource, resource_bounds};
use crate::int::CURVE_COORDINATE_SAFETY_BITS;
use crate::int::{
    CurveInt, CurvePath as IntCurvePath, CurveSegment as IntCurveSegment, CurveShape as IntCurveShape,
};
use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
use alloc::vec::Vec;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterScaleError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

/// A curve resource converted into one integer shape together with the adapter
/// that defines its coordinate system.
///
/// Every path in the source resource becomes a contour of the returned shape.
/// Paths that collapse completely on the selected integer grid are omitted,
/// so the integer shape may be empty.
pub struct CurveConverter<P: FloatPointCompatible, I: CurveInt> {
    adapter: FloatPointAdapter<P, I>,
    shape: IntCurveShape<I>,
    report: CurveConversionReport,
}

/// Observable topology changes made during float-to-integer conversion.
///
/// Coordinate rounding happens for every conversion and is not counted here.
/// This report records only source geometry that disappears or changes segment
/// kind because it cannot be represented on the selected integer grid.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct CurveConversionReport {
    /// Number of source contours examined.
    pub contour_count: usize,
    /// Number of source contours omitted because none of their segments
    /// survived conversion.
    pub collapsed_contour_count: usize,
    /// Number of source segments omitted after snapping reduced them to empty
    /// integer geometry.
    pub collapsed_segment_count: usize,
    /// Number of rational arcs replaced by straight lines because their
    /// ellipse, weights, or direction collapsed on the integer grid.
    pub linearized_arc_count: usize,
}

impl CurveConversionReport {
    /// Returns whether conversion omitted geometry or changed an arc to a line.
    #[inline]
    pub fn has_degeneracies(&self) -> bool {
        self.collapsed_contour_count != 0
            || self.collapsed_segment_count != 0
            || self.linearized_arc_count != 0
    }
}

/// Invalid explicit scale requested for float-to-integer conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CurveConversionError {
    /// Requested scale would exceed the safe integer coordinate range.
    ScaleTooLarge,
    /// Requested scale is zero or negative.
    ScaleNonPositive,
    /// Requested scale is NaN or infinite.
    ScaleNotFinite,
}

impl From<FloatPointAdapterScaleError> for CurveConversionError {
    fn from(error: FloatPointAdapterScaleError) -> Self {
        match error {
            FloatPointAdapterScaleError::ScaleTooLarge => Self::ScaleTooLarge,
            FloatPointAdapterScaleError::ScaleNonPositive => Self::ScaleNonPositive,
            FloatPointAdapterScaleError::ScaleNotFinite => Self::ScaleNotFinite,
        }
    }
}

impl core::fmt::Display for CurveConversionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ScaleTooLarge => formatter.write_str("conversion scale exceeds the safe coordinate range"),
            Self::ScaleNonPositive => formatter.write_str("conversion scale must be positive"),
            Self::ScaleNotFinite => formatter.write_str("conversion scale must be finite"),
        }
    }
}

impl core::error::Error for CurveConversionError {}

impl<P: FloatPointCompatible, I: CurveInt> CurveConverter<P, I> {
    /// Coordinate magnitude bound used by the integer curve kernel.
    ///
    /// Six reserved bits cover the constant growth of cubic polynomial
    /// coefficients. Operations whose width grows with the polynomial degree
    /// use extended-width products separately.
    const COORDINATE_BITS: u32 = I::BITS - CURVE_COORDINATE_SAFETY_BITS;

    /// Chooses the largest safe power-of-two scale for all paths in a curve
    /// resource and converts them immediately.
    pub fn new<R>(source: &R) -> Self
    where
        R: CurveResource<P> + ?Sized,
    {
        let bounds = resource_bounds(source).unwrap_or_else(FloatRect::zero);
        let adapter = FloatPointAdapter::with_coordinate_bits(bounds, Self::COORDINATE_BITS);
        let (shape, report) = convert_resource(source, &adapter);
        Self {
            adapter,
            shape,
            report,
        }
    }

    /// Converts all paths in a curve resource with an explicitly requested scale.
    pub fn try_with_scale<R>(source: &R, scale: P::Scalar) -> Result<Self, CurveConversionError>
    where
        R: CurveResource<P> + ?Sized,
    {
        let bounds = resource_bounds(source).unwrap_or_else(FloatRect::zero);
        let adapter =
            FloatPointAdapter::try_with_scale_and_coordinate_bits(bounds, scale, Self::COORDINATE_BITS)?;
        let (shape, report) = convert_resource(source, &adapter);
        Ok(Self {
            adapter,
            shape,
            report,
        })
    }

    /// Returns the effective float-to-integer conversion scale.
    #[inline]
    pub fn scale(&self) -> P::Scalar {
        self.adapter.dir_scale()
    }

    /// Returns the adapter used to translate between float and integer coordinates.
    #[inline]
    pub fn adapter(&self) -> &FloatPointAdapter<P, I> {
        &self.adapter
    }

    /// Returns the converted integer shape without consuming this converter.
    #[inline]
    pub fn shape(&self) -> &IntCurveShape<I> {
        &self.shape
    }

    /// Returns topology changes observed during conversion.
    #[inline]
    pub fn report(&self) -> CurveConversionReport {
        self.report
    }

    /// Consumes this converter and returns the converted integer shape.
    #[inline]
    pub fn into_shape(self) -> IntCurveShape<I> {
        self.shape
    }

    /// Consumes this converter and returns its adapter and integer shape.
    #[inline]
    pub fn into_parts(self) -> (FloatPointAdapter<P, I>, IntCurveShape<I>) {
        (self.adapter, self.shape)
    }
}

pub(crate) fn convert_resource<P, I, R>(
    source: &R,
    adapter: &FloatPointAdapter<P, I>,
) -> (IntCurveShape<I>, CurveConversionReport)
where
    P: FloatPointCompatible,
    I: CurveInt,
    R: CurveResource<P> + ?Sized,
{
    let mut report = CurveConversionReport::default();
    let mut contours = Vec::new();
    for path in source.iter_paths() {
        report.contour_count += 1;
        if let Some(path) = convert_path(path, adapter, &mut report) {
            contours.push(path);
        } else {
            report.collapsed_contour_count += 1;
        }
    }
    (IntCurveShape { contours }, report)
}

pub(crate) fn convert_shapes_to_float<P: FloatPointCompatible, I: CurveInt>(
    source: Vec<IntCurveShape<I>>,
    adapter: &FloatPointAdapter<P, I>,
) -> Vec<FloatCurveShape<P>> {
    source
        .into_iter()
        .map(|shape| {
            let contours = shape
                .contours
                .into_iter()
                .map(|path| convert_path_to_float(path, adapter))
                .collect();
            FloatCurveShape::from_validated_contours(contours)
        })
        .collect()
}

fn convert_path_to_float<P: FloatPointCompatible, I: CurveInt>(
    source: IntCurvePath<I>,
    adapter: &FloatPointAdapter<P, I>,
) -> FloatCurvePath<P> {
    let start = adapter.int_to_float(&source.start);
    let segments = source
        .segments
        .into_iter()
        .map(|segment| convert_segment_to_float(segment, adapter))
        .collect();
    FloatCurvePath::from_validated_parts(start, segments)
}

fn convert_segment_to_float<P: FloatPointCompatible, I: CurveInt>(
    source: IntCurveSegment<I>,
    adapter: &FloatPointAdapter<P, I>,
) -> FloatCurveSegment<P> {
    match source {
        IntCurveSegment::Line { to } => FloatCurveSegment::Line {
            to: adapter.int_to_float(&to),
        },
        IntCurveSegment::Quad { ctrl, to } => FloatCurveSegment::Quad {
            ctrl: adapter.int_to_float(&ctrl),
            to: adapter.int_to_float(&to),
        },
        IntCurveSegment::Cubic { ctrl0, ctrl1, to } => FloatCurveSegment::Cubic {
            ctrl0: adapter.int_to_float(&ctrl0),
            ctrl1: adapter.int_to_float(&ctrl1),
            to: adapter.int_to_float(&to),
        },
        IntCurveSegment::Arc { arc } => FloatCurveSegment::Arc {
            arc: convert_arc_to_float(arc, adapter),
        },
    }
}

fn convert_arc_to_float<P: FloatPointCompatible, I: CurveInt>(
    source: ArcSegment<I>,
    adapter: &FloatPointAdapter<P, I>,
) -> RationalArc<P> {
    let axis_x_x = adapter.len_to_float(source.ellipse.axis_x.x);
    let axis_x_y = adapter.len_to_float(source.ellipse.axis_x.y);
    let axis_y_x = adapter.len_to_float(source.ellipse.axis_y.x);
    let axis_y_y = adapter.len_to_float(source.ellipse.axis_y.y);
    let start_angle = phase_angle::<P::Scalar, I>(source.start_phase);
    let end_angle = phase_angle::<P::Scalar, I>(source.end_phase);
    let sweep_angle = directed_sweep(start_angle, end_angle, source.direction);
    let denominator = P::Scalar::from_wide_int(FixedScale::<I>::DENOMINATOR);

    RationalArc {
        ellipse: Ellipse {
            center: adapter.int_to_float(&source.ellipse.center),
            radius_x: (axis_x_x * axis_x_x + axis_x_y * axis_x_y).sqrt(),
            radius_y: (axis_y_x * axis_y_x + axis_y_y * axis_y_y).sqrt(),
            rotation: vector_angle(axis_x_x, axis_x_y),
        },
        control_points: source.control_points.map(|point| adapter.int_to_float(&point)),
        weights: source
            .weights
            .map(|weight| P::Scalar::from_int(weight) / denominator),
        start_angle,
        sweep_angle,
    }
}

fn phase_angle<F: FloatNumber, I: CurveInt>(phase: ArcPhase<I>) -> F {
    vector_angle(F::from_int(phase.cos), F::from_int(phase.sin))
}

fn vector_angle<F: FloatNumber>(x: F, y: F) -> F {
    let length = (x * x + y * y).sqrt();
    let cosine = (x / length).max(-F::ONE).min(F::ONE);
    let angle = cosine.acos();
    if y < F::ZERO { -angle } else { angle }
}

fn directed_sweep<F: FloatNumber>(start: F, end: F, direction: ArcDirection) -> F {
    let pi = (-F::ONE).acos();
    let turn = pi * F::TWO;
    let mut sweep = end - start;
    match direction {
        ArcDirection::CounterClockwise => {
            if sweep <= F::ZERO {
                sweep = sweep + turn;
            }
        }
        ArcDirection::Clockwise => {
            if sweep >= F::ZERO {
                sweep = sweep - turn;
            }
        }
    }
    sweep
}

fn convert_path<P: FloatPointCompatible, I: CurveInt>(
    source: &FloatCurvePath<P>,
    adapter: &FloatPointAdapter<P, I>,
    report: &mut CurveConversionReport,
) -> Option<IntCurvePath<I>> {
    let start = adapter.float_to_int(&source.start);
    let mut current = start;
    let mut segments = Vec::with_capacity(source.segments.len());

    for segment in &source.segments {
        match segment {
            FloatCurveSegment::Line { to } => {
                let to = adapter.float_to_int(to);
                if current == to {
                    report.collapsed_segment_count += 1;
                } else {
                    segments.push(IntCurveSegment::Line { to });
                }
                current = to;
            }
            FloatCurveSegment::Quad { ctrl, to } => {
                let ctrl = adapter.float_to_int(ctrl);
                let to = adapter.float_to_int(to);
                if current == to {
                    report.collapsed_segment_count += 1;
                } else {
                    segments.push(IntCurveSegment::Quad { ctrl, to });
                }
                current = to;
            }
            FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => {
                let ctrl0 = adapter.float_to_int(ctrl0);
                let ctrl1 = adapter.float_to_int(ctrl1);
                let to = adapter.float_to_int(to);
                let closed_spike = current == to && (current == ctrl0 || current == ctrl1 || ctrl0 == ctrl1);
                if closed_spike {
                    report.collapsed_segment_count += 1;
                } else {
                    segments.push(IntCurveSegment::Cubic { ctrl0, ctrl1, to });
                }
                current = to;
            }
            FloatCurveSegment::Arc { arc } => {
                current = append_rational_arc(*arc, current, adapter, &mut segments, report);
            }
        }
    }

    if segments.is_empty() {
        None
    } else {
        Some(IntCurvePath { start, segments })
    }
}

fn append_rational_arc<P: FloatPointCompatible, I: CurveInt>(
    arc: RationalArc<P>,
    current: IntPoint<I>,
    adapter: &FloatPointAdapter<P, I>,
    output: &mut Vec<IntCurveSegment<I>>,
    report: &mut CurveConversionReport,
) -> IntPoint<I> {
    let float_frame = FloatEllipseFrame::new(arc.ellipse);
    let ellipse = float_frame.to_int(adapter);
    let direction = if arc.sweep_angle > P::Scalar::ZERO {
        ArcDirection::CounterClockwise
    } else {
        ArcDirection::Clockwise
    };
    let end = adapter.float_to_int(&arc.control_points[2]);
    if current == end {
        report.collapsed_segment_count += 1;
        return end;
    }
    let mut control_point = adapter.float_to_int(&arc.control_points[1]);
    control_point.x = control_point.x.clamp(current.x.min(end.x), current.x.max(end.x));
    control_point.y = control_point.y.clamp(current.y.min(end.y), current.y.max(end.y));
    let fixed_start = FloatArcPhase::from_angle(arc.start_angle).to_fixed::<I>();
    let fixed_end = FloatArcPhase::from_angle(arc.start_angle + arc.sweep_angle).to_fixed::<I>();
    let weights = arc.weights.map(fixed_weight::<P::Scalar, I>);

    if !ellipse_frame_is_valid(&ellipse)
        || weights.iter().any(|weight| *weight <= I::ZERO)
        || !fixed_direction_is_valid(fixed_start, fixed_end, direction)
    {
        report.linearized_arc_count += 1;
        output.push(IntCurveSegment::Line { to: end });
        return end;
    }

    let int_arc = ArcSegment {
        ellipse,
        control_points: [current, control_point, end],
        weights,
        start_phase: fixed_start,
        end_phase: fixed_end,
        direction,
    };
    debug_assert!(
        int_arc.is_xy_monotone(),
        "converted arc control polygon must be XY-monotone"
    );
    int_arc.debug_assert_invariants();
    output.push(IntCurveSegment::Arc { arc: int_arc });
    end
}

fn fixed_weight<F: FloatNumber, I: CurveInt>(weight: F) -> I {
    let denominator = FixedScale::<I>::DENOMINATOR.to_f64();
    I::from_rounded_float(weight.to_f64() * denominator)
}

fn fixed_direction_is_valid<I: CurveInt>(
    start: ArcPhase<I>,
    end: ArcPhase<I>,
    direction: ArcDirection,
) -> bool {
    let cross = start.cos.to_wide() * end.sin.to_wide() - start.sin.to_wide() * end.cos.to_wide();
    match direction {
        ArcDirection::Clockwise => cross < I::Wide::ZERO,
        ArcDirection::CounterClockwise => cross > I::Wide::ZERO,
    }
}

fn ellipse_frame_is_valid<I: CurveInt>(frame: &EllipseFrame<I>) -> bool {
    let axis_x_x = frame.axis_x.x.to_wide();
    let axis_x_y = frame.axis_x.y.to_wide();
    let axis_y_x = frame.axis_y.x.to_wide();
    let axis_y_y = frame.axis_y.y.to_wide();
    let axis_x_is_zero = axis_x_x == I::Wide::ZERO && axis_x_y == I::Wide::ZERO;
    let axis_y_is_zero = axis_y_x == I::Wide::ZERO && axis_y_y == I::Wide::ZERO;
    let determinant = axis_x_x * axis_y_y - axis_x_y * axis_y_x;

    !axis_x_is_zero && !axis_y_is_zero && determinant != I::Wide::ZERO
}

#[derive(Clone, Copy)]
struct FloatArcPhase<F: FloatNumber> {
    cos: F,
    sin: F,
}

impl<F: FloatNumber> FloatArcPhase<F> {
    fn from_angle(angle: F) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::normalized(cos, sin)
    }

    fn normalized(cos: F, sin: F) -> Self {
        let length = (cos * cos + sin * sin).sqrt();
        Self {
            cos: cos / length,
            sin: sin / length,
        }
    }

    fn to_fixed<I: CurveInt>(self) -> ArcPhase<I> {
        let cos = self.cos.to_f64();
        let sin = self.sin.to_f64();
        let length = <f64 as FloatNumber>::sqrt(cos * cos + sin * sin);
        let cos = cos / length;
        let sin = sin / length;
        let denominator = FixedScale::<I>::DENOMINATOR.to_f64();

        if cos.abs() <= sin.abs() {
            let fixed_cos = I::from_rounded_float(cos * denominator);
            let cos = fixed_cos.to_wide();
            let sin = (FixedScale::<I>::DENOMINATOR * FixedScale::<I>::DENOMINATOR - cos * cos).isqrt();
            ArcPhase {
                cos: fixed_cos,
                sin: I::from_wide(if self.sin < F::ZERO { -sin } else { sin }),
            }
        } else {
            let fixed_sin = I::from_rounded_float(sin * denominator);
            let sin = fixed_sin.to_wide();
            let cos = (FixedScale::<I>::DENOMINATOR * FixedScale::<I>::DENOMINATOR - sin * sin).isqrt();
            ArcPhase {
                cos: I::from_wide(if self.cos < F::ZERO { -cos } else { cos }),
                sin: fixed_sin,
            }
        }
    }
}

struct FloatEllipseFrame<P: FloatPointCompatible> {
    center: P,
    axis_x_x: P::Scalar,
    axis_x_y: P::Scalar,
    axis_y_x: P::Scalar,
    axis_y_y: P::Scalar,
}

impl<P: FloatPointCompatible> FloatEllipseFrame<P> {
    fn new(ellipse: Ellipse<P>) -> Self {
        let (rotation_sin, rotation_cos) = ellipse.rotation.sin_cos();
        Self {
            center: ellipse.center,
            axis_x_x: ellipse.radius_x * rotation_cos,
            axis_x_y: ellipse.radius_x * rotation_sin,
            axis_y_x: -ellipse.radius_y * rotation_sin,
            axis_y_y: ellipse.radius_y * rotation_cos,
        }
    }

    fn to_int<I: CurveInt>(&self, adapter: &FloatPointAdapter<P, I>) -> EllipseFrame<I> {
        EllipseFrame {
            center: adapter.float_to_int(&self.center),
            axis_x: ArcVector {
                x: adapter.round_len_to_int(self.axis_x_x),
                y: adapter.round_len_to_int(self.axis_x_y),
            },
            axis_y: ArcVector {
                x: adapter.round_len_to_int(self.axis_y_x),
                y: adapter.round_len_to_int(self.axis_y_y),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float::curve::arc::{Ellipse, EllipticArc};
    use crate::float::curve::builder::{CurveBuilder, CurveError};
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;
    use i_overlay::i_shape::int::IntPoint;

    fn assert_arc_control_polygons_are_monotone<I: CurveInt>(shape: &IntCurveShape<I>) {
        for contour in &shape.contours {
            for segment in &contour.segments {
                if let IntCurveSegment::Arc { arc } = segment {
                    assert!(arc.is_xy_monotone());
                }
            }
        }
    }

    fn float_shape() -> Result<FloatCurveShape<[f64; 2]>, CurveError> {
        CurveBuilder::new()
            .move_to([0.0, 0.0])?
            .quad_to([5.0, 10.0], [10.0, 0.0])?
            .line_to([0.0, 0.0])?
            .build()
    }

    fn arc_shape(
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        sweep_angle: f64,
    ) -> Result<FloatCurveShape<[f64; 2]>, CurveError> {
        let arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0, 0.0],
                radius_x,
                radius_y,
                rotation,
            },
            start_angle,
            sweep_angle,
        };
        CurveBuilder::new()
            .move_to(arc.start_point())?
            .arc_to(arc)?
            .close_contour()?
            .build()
    }

    #[test]
    fn automatically_selects_adapter_and_converts_all_points() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::new(&float_shape()?);
        let shape = converter.shape();

        assert_eq!(shape.contours.len(), 1);
        assert_eq!(shape.contours[0].segments.len(), 2);
        assert!(converter.adapter().rect().contains(&[5.0, 10.0]));
        assert!(matches!(
            shape.contours[0].segments[0],
            IntCurveSegment::Quad { .. }
        ));
        assert_eq!(
            converter.report(),
            CurveConversionReport {
                contour_count: 1,
                ..Default::default()
            }
        );
        Ok(())
    }

    #[test]
    fn automatic_adapter_reserves_six_coordinate_bits() -> Result<(), CurveError> {
        let source = float_shape()?;
        let i16_converter = CurveConverter::<_, i16>::new(&source);
        let i32_converter = CurveConverter::<_, i32>::new(&source);
        let i64_converter = CurveConverter::<_, i64>::new(&source);

        assert_eq!(i16_converter.scale(), 2_f64.powi(7));
        assert_eq!(i32_converter.scale(), 2_f64.powi(23));
        assert_eq!(i64_converter.scale(), 2_f64.powi(55));
        Ok(())
    }

    #[test]
    fn requested_scale_is_used_for_conversion() -> Result<(), CurveError> {
        let converter =
            CurveConverter::<_, i32>::try_with_scale(&float_shape()?, 10.0).expect("scale must fit");
        let shape = converter.shape();

        assert_eq!(converter.scale(), 10.0);
        assert_eq!(shape.contours[0].start, IntPoint::new(-50, -50));
        match shape.contours[0].segments[0] {
            IntCurveSegment::Quad { ctrl, to } => {
                assert_eq!(ctrl, IntPoint::new(0, 50));
                assert_eq!(to, IntPoint::new(50, -50));
            }
            _ => panic!("expected quadratic segment"),
        }
        Ok(())
    }

    #[test]
    fn requested_scale_respects_curve_coordinate_bits() -> Result<(), CurveError> {
        let error = match CurveConverter::<_, i32>::try_with_scale(&float_shape()?, 2_f64.powi(24)) {
            Ok(_) => panic!("scale above the 26-bit curve range must fail"),
            Err(error) => error,
        };

        assert_eq!(error, CurveConversionError::ScaleTooLarge);
        Ok(())
    }

    #[test]
    fn requested_scale_reports_adapter_errors() -> Result<(), CurveError> {
        let error = match CurveConverter::<_, i32>::try_with_scale(&float_shape()?, 0.0) {
            Ok(_) => panic!("zero scale must fail"),
            Err(error) => error,
        };

        assert_eq!(error, CurveConversionError::ScaleNonPositive);

        let error = match CurveConverter::<_, i32>::try_with_scale(&float_shape()?, 1.0e20) {
            Ok(_) => panic!("unsafe scale must fail"),
            Err(error) => error,
        };
        assert_eq!(error, CurveConversionError::ScaleTooLarge);
        Ok(())
    }

    #[test]
    fn into_parts_preserves_adapter_and_shape() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::new(&float_shape()?);
        let (adapter, shape) = converter.into_parts();

        assert_eq!(shape.contours.len(), 1);
        assert!(adapter.rect().contains(&[0.0, 0.0]));
        Ok(())
    }

    #[test]
    fn full_circle_is_split_at_four_extrema() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            &arc_shape(10.0, 10.0, 0.0, 0.0, core::f64::consts::TAU)?,
            1.0,
        )
        .expect("scale must fit");
        let segments = &converter.shape().contours[0].segments;

        assert_eq!(segments.len(), 4);
        let expected_ends = [
            IntPoint::new(0, 10),
            IntPoint::new(-10, 0),
            IntPoint::new(0, -10),
            IntPoint::new(10, 0),
        ];
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let middle_weight =
            (FixedScale::<i32>::DENOMINATOR as f64 * core::f64::consts::FRAC_1_SQRT_2).round() as i32;

        for (segment, expected_end) in segments.iter().zip(expected_ends) {
            let IntCurveSegment::Arc { arc } = segment else {
                panic!("expected arc segment");
            };
            assert_eq!(arc.control_points[2], expected_end);
            assert_eq!(arc.weights, [one, middle_weight, one]);
            assert_eq!(arc.direction, ArcDirection::CounterClockwise);
        }
        for pair in segments.windows(2) {
            let IntCurveSegment::Arc { arc: first } = &pair[0] else {
                panic!("expected arc segment");
            };
            let IntCurveSegment::Arc { arc: second } = &pair[1] else {
                panic!("expected arc segment");
            };
            assert_eq!(first.control_points[2], second.control_points[0]);
        }

        let IntCurveSegment::Arc { arc } = &segments[0] else {
            panic!("expected arc segment");
        };
        assert_eq!(
            arc.control_points,
            [IntPoint::new(10, 0), IntPoint::new(10, 10), IntPoint::new(0, 10)]
        );
        Ok(())
    }

    #[test]
    fn partial_arc_is_split_only_at_internal_extrema() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            &arc_shape(
                10.0,
                10.0,
                0.0,
                core::f64::consts::FRAC_PI_4,
                core::f64::consts::FRAC_PI_2,
            )?,
            100.0,
        )
        .expect("scale must fit");
        let segments = &converter.shape().contours[0].segments;
        let arcs = segments
            .iter()
            .filter(|segment| matches!(segment, IntCurveSegment::Arc { .. }))
            .count();

        assert_eq!(arcs, 2);
        assert!(matches!(segments.last(), Some(IntCurveSegment::Line { .. })));
        Ok(())
    }

    #[test]
    fn clockwise_circle_preserves_direction_and_order() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            &arc_shape(10.0, 10.0, 0.0, 0.0, -core::f64::consts::TAU)?,
            1.0,
        )
        .expect("scale must fit");
        let segments = &converter.shape().contours[0].segments;
        let expected_ends = [
            IntPoint::new(0, -10),
            IntPoint::new(-10, 0),
            IntPoint::new(0, 10),
            IntPoint::new(10, 0),
        ];

        assert_eq!(segments.len(), 4);
        for (segment, expected_end) in segments.iter().zip(expected_ends) {
            let IntCurveSegment::Arc { arc } = segment else {
                panic!("expected arc segment");
            };
            assert_eq!(arc.control_points[2], expected_end);
            assert_eq!(arc.direction, ArcDirection::Clockwise);
        }
        Ok(())
    }

    #[test]
    fn rotated_ellipse_frame_is_converted_as_vectors() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            &arc_shape(
                10.0,
                5.0,
                core::f64::consts::FRAC_PI_2,
                0.0,
                core::f64::consts::TAU,
            )?,
            1.0,
        )
        .expect("scale must fit");
        let first = converter.shape().contours[0]
            .segments
            .iter()
            .find_map(|segment| match segment {
                IntCurveSegment::Arc { arc } => Some(arc),
                _ => None,
            })
            .expect("expected arc segment");

        assert_eq!(first.ellipse.center, IntPoint::new(0, 0));
        assert_eq!(first.ellipse.axis_x, ArcVector { x: 0, y: 10 });
        assert_eq!(first.ellipse.axis_y, ArcVector { x: -5, y: 0 });
        Ok(())
    }

    #[test]
    fn rotated_ellipse_is_split_at_world_extrema() -> Result<(), CurveError> {
        let radius_x = 10.0;
        let radius_y = 5.0;
        let rotation = 0.4;
        let scale = 100.0;
        let converter = CurveConverter::<_, i32>::try_with_scale(
            &arc_shape(radius_x, radius_y, rotation, 0.2, core::f64::consts::TAU)?,
            scale,
        )
        .expect("scale must fit");
        let contour = &converter.shape().contours[0];
        let mut endpoints = Vec::with_capacity(contour.segments.len() + 1);
        endpoints.push(contour.start);
        for segment in &contour.segments {
            if let IntCurveSegment::Arc { arc } = segment {
                endpoints.push(arc.control_points[2]);
            }
        }

        let min_x = endpoints.iter().map(|point| point.x).min().unwrap();
        let max_x = endpoints.iter().map(|point| point.x).max().unwrap();
        let min_y = endpoints.iter().map(|point| point.y).min().unwrap();
        let max_y = endpoints.iter().map(|point| point.y).max().unwrap();
        let (rotation_sin, rotation_cos) = rotation.sin_cos();
        let extent_x = ((radius_x * rotation_cos).powi(2) + (radius_y * rotation_sin).powi(2)).sqrt();
        let extent_y = ((radius_x * rotation_sin).powi(2) + (radius_y * rotation_cos).powi(2)).sqrt();

        assert_eq!(min_x, (-extent_x * scale).round() as i32);
        assert_eq!(max_x, (extent_x * scale).round() as i32);
        assert_eq!(min_y, (-extent_y * scale).round() as i32);
        assert_eq!(max_y, (extent_y * scale).round() as i32);
        Ok(())
    }

    #[test]
    fn collapsed_ellipse_frame_falls_back_to_lines() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            &arc_shape(10.0, 0.1, 0.0, 0.0, core::f64::consts::TAU)?,
            1.0,
        )
        .expect("scale must fit");
        let segments = &converter.shape().contours[0].segments;

        assert!(!segments.is_empty());
        assert!(
            segments
                .iter()
                .all(|segment| matches!(segment, IntCurveSegment::Line { .. }))
        );
        assert!(converter.report().linearized_arc_count > 0);
        assert!(converter.report().has_degeneracies());
        Ok(())
    }

    #[test]
    fn fully_collapsed_arc_contour_is_removed() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            &arc_shape(0.1, 0.1, 0.0, 0.0, core::f64::consts::TAU)?,
            1.0,
        )
        .expect("scale must fit");

        assert!(converter.shape().contours.is_empty());
        assert_eq!(converter.report().contour_count, 1);
        assert_eq!(converter.report().collapsed_contour_count, 1);
        assert!(converter.report().collapsed_segment_count > 0);
        assert!(converter.report().has_degeneracies());
        Ok(())
    }

    #[test]
    fn converts_varied_ellipses_without_leaving_adapter_bounds() -> Result<(), CurveError> {
        let radii = [0.1, 1.0, 10.0, 100.0];
        let rotations = [0.0, 0.3, 1.2];
        let starts = [0.1, 1.0, 3.0];
        let sweeps = [0.2, 1.5, 3.0, 5.9];

        for radius_x in radii {
            for radius_y in radii {
                for rotation in rotations {
                    for start in starts {
                        for sweep in sweeps {
                            for direction in [-1.0, 1.0] {
                                let shape =
                                    arc_shape(radius_x, radius_y, rotation, start, direction * sweep)?;
                                let converter = CurveConverter::<_, i32>::new(&shape);
                                assert!(!converter.shape().contours[0].segments.is_empty());
                                assert_arc_control_polygons_are_monotone(converter.shape());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn accepts_f32_full_revolution() -> Result<(), CurveError> {
        let sweep = core::f32::consts::TAU;
        let arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0_f32, 0.0],
                radius_x: 10.0,
                radius_y: 5.0,
                rotation: 0.4,
            },
            start_angle: 0.2,
            sweep_angle: sweep,
        };
        let shape = CurveBuilder::new()
            .move_to(arc.start_point())?
            .arc_to(arc)?
            .close_contour()?
            .build()?;
        let converter = CurveConverter::<_, i32>::new(&shape);
        let arc_count = converter.shape().contours[0]
            .segments
            .iter()
            .filter(|segment| matches!(segment, IntCurveSegment::Arc { .. }))
            .count();

        assert!(arc_count >= 4);
        Ok(())
    }

    #[test]
    fn converts_arc_for_all_integer_widths() -> Result<(), CurveError> {
        let source = arc_shape(10.0, 5.0, 0.37, 0.21, core::f64::consts::TAU)?;
        let i16_shape = CurveConverter::<_, i16>::new(&source);
        let i32_shape = CurveConverter::<_, i32>::new(&source);
        let i64_shape = CurveConverter::<_, i64>::new(&source);

        assert!(!i16_shape.shape().contours[0].segments.is_empty());
        assert!(!i32_shape.shape().contours[0].segments.is_empty());
        assert!(!i64_shape.shape().contours[0].segments.is_empty());
        assert_arc_control_polygons_are_monotone(i16_shape.shape());
        assert_arc_control_polygons_are_monotone(i32_shape.shape());
        assert_arc_control_polygons_are_monotone(i64_shape.shape());
        Ok(())
    }

    #[test]
    fn preserves_general_rational_arc_weights() -> Result<(), CurveError> {
        let source_arc = EllipticArc {
            ellipse: Ellipse {
                center: [0.0, 0.0],
                radius_x: 10.0,
                radius_y: 5.0,
                rotation: 0.0,
            },
            start_angle: 0.0,
            sweep_angle: core::f64::consts::FRAC_PI_2,
        };
        let mut rational = source_arc.to_rational_arcs()?.remove(0);
        rational.weights = [0.75, 0.5, 0.875];
        let start = rational.start_point();
        let shape = CurveBuilder::new()
            .move_to(start)?
            .rational_arc_to(rational)?
            .line_to(start)?
            .build()?;

        let converter = CurveConverter::<_, i32>::try_with_scale(&shape, 100.0).expect("scale must fit");
        let IntCurveSegment::Arc { arc } = &converter.shape().contours[0].segments[0] else {
            panic!("expected rational arc");
        };

        assert_eq!(
            arc.weights,
            [
                fixed_weight::<f64, i32>(0.75),
                fixed_weight::<f64, i32>(0.5),
                fixed_weight::<f64, i32>(0.875),
            ]
        );
        Ok(())
    }
}
