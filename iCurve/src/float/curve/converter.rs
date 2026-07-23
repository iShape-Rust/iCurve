use crate::float::curve::arc::{Ellipse, EllipticArc};
use crate::float::curve::path::CurvePath as FloatCurvePath;
use crate::float::curve::segment::CurveSegment as FloatCurveSegment;
use crate::float::curve::shape::CurveShape as FloatCurveShape;
use crate::int::curve::path::CurvePath as IntCurvePath;
use crate::int::curve::segment::CurveSegment as IntCurveSegment;
use crate::int::curve::shape::CurveShape as IntCurveShape;
use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
use alloc::vec::Vec;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterScaleError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

/// Converted integer contours together with the adapter that defines their
/// coordinate system.
pub struct CurveConverter<P: FloatPointCompatible, I: IntNumber> {
    adapter: FloatPointAdapter<P, I>,
    shape: IntCurveShape<I>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveConversionError {
    Adapter(FloatPointAdapterScaleError),
}

impl From<FloatPointAdapterScaleError> for CurveConversionError {
    fn from(error: FloatPointAdapterScaleError) -> Self {
        Self::Adapter(error)
    }
}

impl<P: FloatPointCompatible, I: IntNumber> CurveConverter<P, I> {
    /// Chooses the largest safe power-of-two scale for all source contours and
    /// converts them immediately.
    pub fn new(source: FloatCurveShape<P>) -> Self {
        let adapter = FloatPointAdapter::new(source.bounds());
        let shape = convert_shape(source, &adapter);
        Self { adapter, shape }
    }

    /// Converts all source contours with an explicitly requested scale.
    pub fn try_with_scale(
        source: FloatCurveShape<P>,
        scale: P::Scalar,
    ) -> Result<Self, CurveConversionError> {
        let adapter = FloatPointAdapter::try_with_scale(source.bounds(), scale)?;
        let shape = convert_shape(source, &adapter);
        Ok(Self { adapter, shape })
    }

    #[inline]
    pub fn adapter(&self) -> &FloatPointAdapter<P, I> {
        &self.adapter
    }

    #[inline]
    pub fn shape(&self) -> &IntCurveShape<I> {
        &self.shape
    }

    #[inline]
    pub fn into_shape(self) -> IntCurveShape<I> {
        self.shape
    }

    #[inline]
    pub fn into_parts(self) -> (FloatPointAdapter<P, I>, IntCurveShape<I>) {
        (self.adapter, self.shape)
    }
}

fn convert_shape<P: FloatPointCompatible, I: IntNumber>(
    source: FloatCurveShape<P>,
    adapter: &FloatPointAdapter<P, I>,
) -> IntCurveShape<I> {
    let contours = source
        .contours
        .into_iter()
        .filter_map(|path| convert_path(path, adapter))
        .collect();
    IntCurveShape { contours }
}

fn convert_path<P: FloatPointCompatible, I: IntNumber>(
    source: FloatCurvePath<P>,
    adapter: &FloatPointAdapter<P, I>,
) -> Option<IntCurvePath<I>> {
    let start = adapter.float_to_int(&source.start);
    let mut current = start;
    let mut segments = Vec::with_capacity(source.segments.len());

    for segment in source.segments {
        match segment {
            FloatCurveSegment::Line { to } => {
                let to = adapter.float_to_int(&to);
                segments.push(IntCurveSegment::Line { to });
                current = to;
            }
            FloatCurveSegment::Quad { ctrl, to } => {
                let ctrl = adapter.float_to_int(&ctrl);
                let to = adapter.float_to_int(&to);
                segments.push(IntCurveSegment::Quad { ctrl, to });
                current = to;
            }
            FloatCurveSegment::Cubic { ctrl0, ctrl1, to } => {
                let ctrl0 = adapter.float_to_int(&ctrl0);
                let ctrl1 = adapter.float_to_int(&ctrl1);
                let to = adapter.float_to_int(&to);
                segments.push(IntCurveSegment::Cubic { ctrl0, ctrl1, to });
                current = to;
            }
            FloatCurveSegment::Arc { arc } => {
                current = append_arc_segments(arc, current, adapter, &mut segments);
            }
        }
    }

    if segments.is_empty() {
        None
    } else {
        Some(IntCurvePath { start, segments })
    }
}

fn append_arc_segments<P: FloatPointCompatible, I: IntNumber>(
    arc: EllipticArc<P>,
    mut current: IntPoint<I>,
    adapter: &FloatPointAdapter<P, I>,
    output: &mut Vec<IntCurveSegment<I>>,
) -> IntPoint<I> {
    let float_frame = FloatEllipseFrame::new(arc.ellipse);
    let ellipse = float_frame.to_int(adapter);
    let direction = if arc.sweep_angle > P::Scalar::ZERO {
        ArcDirection::CounterClockwise
    } else {
        ArcDirection::Clockwise
    };
    let cuts = collect_arc_cuts(&arc, &float_frame);
    let frame_is_valid = ellipse_frame_is_valid(&ellipse);

    for index in 0..cuts.len() - 1 {
        let start_cut = cuts[index];
        let end_cut = cuts[index + 1];
        let end = if index + 2 == cuts.len() {
            adapter.float_to_int(&arc.end_point())
        } else {
            adapter.float_to_int(&float_frame.point_at(end_cut.phase))
        };

        if current == end {
            current = end;
            continue;
        }

        let segment = if frame_is_valid {
            convert_arc_piece(
                &float_frame,
                ellipse,
                start_cut.phase,
                end_cut.phase,
                direction,
                current,
                end,
                adapter,
            )
        } else {
            None
        };

        match segment {
            Some(segment) => output.push(segment),
            None => output.push(IntCurveSegment::Line { to: end }),
        }
        current = end;
    }

    current
}

#[allow(clippy::too_many_arguments)]
fn convert_arc_piece<P: FloatPointCompatible, I: IntNumber>(
    float_frame: &FloatEllipseFrame<P>,
    ellipse: EllipseFrame<I>,
    start: FloatArcPhase<P::Scalar>,
    end: FloatArcPhase<P::Scalar>,
    direction: ArcDirection,
    start_point: IntPoint<I>,
    end_point: IntPoint<I>,
    adapter: &FloatPointAdapter<P, I>,
) -> Option<IntCurveSegment<I>> {
    let dot = start.dot(end).max(-P::Scalar::ONE).min(P::Scalar::ONE);
    let denominator = P::Scalar::ONE + dot;
    if denominator <= P::Scalar::ZERO {
        return None;
    }

    let weight = (denominator / P::Scalar::TWO).sqrt();
    let control_phase = FloatArcPhase {
        cos: (start.cos + end.cos) / denominator,
        sin: (start.sin + end.sin) / denominator,
    };
    let mut control_point = adapter.float_to_int(&float_frame.point_at(control_phase));
    control_point.x = control_point
        .x
        .clamp(start_point.x.min(end_point.x), start_point.x.max(end_point.x));
    control_point.y = control_point
        .y
        .clamp(start_point.y.min(end_point.y), start_point.y.max(end_point.y));
    let fixed_start = start.to_fixed::<I>();
    let fixed_end = end.to_fixed::<I>();
    let fixed_weight = fixed_weight::<P::Scalar, I>(weight);

    if fixed_weight <= I::ZERO || !fixed_direction_is_valid(fixed_start, fixed_end, direction) {
        return None;
    }

    let one = I::from_wide(FixedScale::<I>::DENOMINATOR);
    let arc = ArcSegment {
        ellipse,
        control_points: [start_point, control_point, end_point],
        weights: [one, fixed_weight, one],
        start_phase: fixed_start,
        end_phase: fixed_end,
        direction,
    };
    debug_assert!(
        arc.is_xy_monotone(),
        "converted arc control polygon must be XY-monotone"
    );
    arc.debug_assert_invariants();

    Some(IntCurveSegment::Arc { arc })
}

fn fixed_weight<F: FloatNumber, I: IntNumber>(weight: F) -> I {
    let denominator = FixedScale::<I>::DENOMINATOR.to_f64();
    I::from_rounded_float(weight.to_f64() * denominator)
}

fn fixed_direction_is_valid<I: IntNumber>(
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

fn ellipse_frame_is_valid<I: IntNumber>(frame: &EllipseFrame<I>) -> bool {
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
struct ArcCut<F: FloatNumber> {
    progress: f64,
    phase: FloatArcPhase<F>,
}

fn collect_arc_cuts<P: FloatPointCompatible>(
    arc: &EllipticArc<P>,
    frame: &FloatEllipseFrame<P>,
) -> Vec<ArcCut<P::Scalar>> {
    let start = FloatArcPhase::from_angle(arc.start_angle);
    let end = if arc.is_full_turn() {
        start
    } else {
        FloatArcPhase::from_angle(arc.start_angle + arc.sweep_angle)
    };
    let counter_clockwise = arc.sweep_angle > P::Scalar::ZERO;
    let sweep = arc.sweep_angle.to_f64().abs();
    let tolerance = if P::Scalar::BITS <= 32 { 1.0e-5 } else { 1.0e-12 };
    let mut cuts = Vec::with_capacity(6);
    cuts.push(ArcCut {
        progress: 0.0,
        phase: start,
    });

    for phase in frame.extremum_phases() {
        let progress = directed_progress(start, phase, counter_clockwise);
        if progress > tolerance && progress < sweep - tolerance {
            cuts.push(ArcCut { progress, phase });
        }
    }

    cuts[1..].sort_by(|a, b| {
        a.progress
            .partial_cmp(&b.progress)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    cuts.dedup_by(|a, b| (a.progress - b.progress).abs() <= tolerance);
    cuts.push(ArcCut {
        progress: sweep,
        phase: end,
    });
    cuts
}

fn directed_progress<F: FloatNumber>(
    start: FloatArcPhase<F>,
    end: FloatArcPhase<F>,
    counter_clockwise: bool,
) -> f64 {
    let dot = start.dot(end).max(-F::ONE).min(F::ONE);
    let angle = dot.acos().to_f64();
    let oriented_cross = if counter_clockwise {
        start.cross(end)
    } else {
        -start.cross(end)
    };

    if oriented_cross < F::ZERO {
        core::f64::consts::TAU - angle
    } else {
        angle
    }
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

    fn opposite(self) -> Self {
        Self {
            cos: -self.cos,
            sin: -self.sin,
        }
    }

    fn dot(self, other: Self) -> F {
        self.cos * other.cos + self.sin * other.sin
    }

    fn cross(self, other: Self) -> F {
        self.cos * other.sin - self.sin * other.cos
    }

    fn to_fixed<I: IntNumber>(self) -> ArcPhase<I> {
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

    fn point_at(&self, phase: FloatArcPhase<P::Scalar>) -> P {
        let x = self.center.x() + self.axis_x_x * phase.cos + self.axis_y_x * phase.sin;
        let y = self.center.y() + self.axis_x_y * phase.cos + self.axis_y_y * phase.sin;
        P::from_xy(x, y)
    }

    fn extremum_phases(&self) -> [FloatArcPhase<P::Scalar>; 4] {
        let x = FloatArcPhase::normalized(self.axis_x_x, self.axis_y_x);
        let y = FloatArcPhase::normalized(self.axis_x_y, self.axis_y_y);
        [x, x.opposite(), y, y.opposite()]
    }

    fn to_int<I: IntNumber>(&self, adapter: &FloatPointAdapter<P, I>) -> EllipseFrame<I> {
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
    use i_overlay::i_float::adapter::FloatPointAdapterScaleError;
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;
    use i_overlay::i_shape::int::IntPoint;

    fn assert_arc_control_polygons_are_monotone<I: IntNumber>(shape: &IntCurveShape<I>) {
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
        let converter = CurveConverter::<_, i32>::new(float_shape()?);
        let shape = converter.shape();

        assert_eq!(shape.contours.len(), 1);
        assert_eq!(shape.contours[0].segments.len(), 2);
        assert!(converter.adapter().rect().contains(&[5.0, 10.0]));
        assert!(matches!(
            shape.contours[0].segments[0],
            IntCurveSegment::Quad { .. }
        ));
        Ok(())
    }

    #[test]
    fn requested_scale_is_used_for_conversion() -> Result<(), CurveError> {
        let converter =
            CurveConverter::<_, i32>::try_with_scale(float_shape()?, 10.0).expect("scale must fit");
        let shape = converter.shape();

        assert_eq!(converter.adapter().dir_scale(), 10.0);
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
    fn requested_scale_reports_adapter_errors() -> Result<(), CurveError> {
        let error = match CurveConverter::<_, i32>::try_with_scale(float_shape()?, 0.0) {
            Ok(_) => panic!("zero scale must fail"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            CurveConversionError::Adapter(FloatPointAdapterScaleError::ScaleNonPositive)
        );

        let error = match CurveConverter::<_, i32>::try_with_scale(float_shape()?, 1.0e20) {
            Ok(_) => panic!("unsafe scale must fail"),
            Err(error) => error,
        };
        assert_eq!(
            error,
            CurveConversionError::Adapter(FloatPointAdapterScaleError::ScaleTooLarge)
        );
        Ok(())
    }

    #[test]
    fn into_parts_preserves_adapter_and_shape() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::new(float_shape()?);
        let (adapter, shape) = converter.into_parts();

        assert_eq!(shape.contours.len(), 1);
        assert!(adapter.rect().contains(&[0.0, 0.0]));
        Ok(())
    }

    #[test]
    fn full_circle_is_split_at_four_extrema() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            arc_shape(10.0, 10.0, 0.0, 0.0, core::f64::consts::TAU)?,
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
            arc_shape(
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
            arc_shape(10.0, 10.0, 0.0, 0.0, -core::f64::consts::TAU)?,
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
            arc_shape(
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
            arc_shape(radius_x, radius_y, rotation, 0.2, core::f64::consts::TAU)?,
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
            arc_shape(10.0, 0.1, 0.0, 0.0, core::f64::consts::TAU)?,
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
        Ok(())
    }

    #[test]
    fn fully_collapsed_arc_contour_is_removed() -> Result<(), CurveError> {
        let converter = CurveConverter::<_, i32>::try_with_scale(
            arc_shape(0.1, 0.1, 0.0, 0.0, core::f64::consts::TAU)?,
            1.0,
        )
        .expect("scale must fit");

        assert!(converter.shape().contours.is_empty());
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
                                let converter = CurveConverter::<_, i32>::new(shape);
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
        let converter = CurveConverter::<_, i32>::new(shape);
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
        let i16_shape = CurveConverter::<_, i16>::new(source.clone());
        let i32_shape = CurveConverter::<_, i32>::new(source.clone());
        let i64_shape = CurveConverter::<_, i64>::new(source);

        assert!(!i16_shape.shape().contours[0].segments.is_empty());
        assert!(!i32_shape.shape().contours[0].segments.is_empty());
        assert!(!i64_shape.shape().contours[0].segments.is_empty());
        assert_arc_control_polygons_are_monotone(i16_shape.shape());
        assert_arc_control_polygons_are_monotone(i32_shape.shape());
        assert_arc_control_polygons_are_monotone(i64_shape.shape());
        Ok(())
    }
}
