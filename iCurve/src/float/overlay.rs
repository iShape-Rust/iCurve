use crate::float::curve::converter::{convert_resource, convert_shapes_to_float};
use crate::float::curve::path::CurvePath;
use crate::float::curve::shape::CurveShape;
use crate::float::resource::CurveResource;
use crate::int::CURVE_COORDINATE_SAFETY_BITS;
use crate::int::{CurveInt, CurveOverlayOptions, CurveOverlayOptionsError, IntCurveOverlay};
use crate::{CurveConversionError, FillRule, OverlayRule, Solver};
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;

/// Curve approximation options expressed in float input coordinates.
///
/// Construct this non-exhaustive configuration from [`Default`] and override
/// only the values your application needs.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct FloatCurveOverlayOptions<F: FloatNumber> {
    /// Absolute chord length below which a curve segment is accepted without
    /// further subdivision.
    ///
    /// `None` preserves the scale-relative default used by the integer engine.
    pub min_chord_length: Option<F>,
    /// Maximum dimensionless sine deviation used to classify a curve segment
    /// as nearly linear. The value must be finite and in the range `(0, 1]`.
    pub angle_tolerance: F,
    /// Hard safety limit for local approximation subdivision. The value must
    /// not exceed [`CurveOverlayOptions::MAX_APPROXIMATION_DEPTH`].
    pub max_approximation_depth: u32,
}

impl<F: FloatNumber> Default for FloatCurveOverlayOptions<F> {
    fn default() -> Self {
        Self {
            min_chord_length: None,
            angle_tolerance: F::from_float(0.125_f64),
            max_approximation_depth: CurveOverlayOptions::default().max_approximation_depth,
        }
    }
}

/// Invalid [`FloatCurveOverlayOptions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FloatCurveOverlayOptionsError {
    /// The requested minimum chord length is zero or negative.
    MinChordLengthNonPositive,
    /// The requested minimum chord length is NaN or infinite.
    MinChordLengthNotFinite,
    /// The requested angle tolerance is NaN or infinite.
    AngleToleranceNotFinite,
    /// The requested angle tolerance is outside `(0, 1]`.
    AngleToleranceOutOfRange,
    /// Integer approximation limits rejected the converted configuration.
    Approximation(CurveOverlayOptionsError),
}

impl From<CurveOverlayOptionsError> for FloatCurveOverlayOptionsError {
    fn from(error: CurveOverlayOptionsError) -> Self {
        Self::Approximation(error)
    }
}

impl core::fmt::Display for FloatCurveOverlayOptionsError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MinChordLengthNonPositive => formatter.write_str("minimum chord length must be positive"),
            Self::MinChordLengthNotFinite => formatter.write_str("minimum chord length must be finite"),
            Self::AngleToleranceNotFinite => formatter.write_str("angle tolerance must be finite"),
            Self::AngleToleranceOutOfRange => {
                formatter.write_str("angle tolerance must be in the range (0, 1]")
            }
            Self::Approximation(_) => formatter.write_str("invalid curve approximation options"),
        }
    }
}

impl core::error::Error for FloatCurveOverlayOptionsError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Approximation(error) => Some(error),
            _ => None,
        }
    }
}

impl<F: FloatNumber> FloatCurveOverlayOptions<F> {
    /// Sets the absolute minimum accepted chord length.
    #[must_use]
    pub fn with_min_chord_length(mut self, length: F) -> Self {
        self.min_chord_length = Some(length);
        self
    }

    /// Sets the maximum sine deviation used for near-linear classification.
    #[must_use]
    pub fn with_angle_tolerance(mut self, tolerance: F) -> Self {
        self.angle_tolerance = tolerance;
        self
    }

    /// Sets the local approximation subdivision limit.
    #[must_use]
    pub fn with_max_approximation_depth(mut self, depth: u32) -> Self {
        self.max_approximation_depth = depth;
        self
    }

    fn to_int<P, I>(
        self,
        adapter: &FloatPointAdapter<P, I>,
    ) -> Result<CurveOverlayOptions, FloatCurveOverlayOptionsError>
    where
        P: FloatPointCompatible<Scalar = F>,
        I: IntNumber,
    {
        let min_chord_length_power = match self.min_chord_length {
            Some(length) => {
                if !length.to_f64().is_finite() {
                    return Err(FloatCurveOverlayOptionsError::MinChordLengthNotFinite);
                }
                if length <= F::ZERO {
                    return Err(FloatCurveOverlayOptionsError::MinChordLengthNonPositive);
                }

                let grid_log2 = length.log2() + adapter.dir_scale().log2();
                if grid_log2 <= F::ZERO {
                    0
                } else {
                    grid_log2.to_i32() as u32
                }
            }
            None => CurveOverlayOptions::default().min_chord_length_power,
        };

        if !self.angle_tolerance.to_f64().is_finite() {
            return Err(FloatCurveOverlayOptionsError::AngleToleranceNotFinite);
        }
        if self.angle_tolerance <= F::ZERO || self.angle_tolerance > F::ONE {
            return Err(FloatCurveOverlayOptionsError::AngleToleranceOutOfRange);
        }

        let angle_power = -self.angle_tolerance.log2();
        let truncated_power = angle_power.to_i32();
        let angle_tolerance_power = if F::from_int(truncated_power) < angle_power {
            truncated_power + 1
        } else {
            truncated_power
        } as u32;

        Ok(CurveOverlayOptions {
            min_chord_length_power,
            angle_tolerance_power,
            max_approximation_depth: self.max_approximation_depth,
        })
    }
}

/// Boolean overlay for float curve shapes.
///
/// Inputs are mapped to one automatically selected fixed-point grid. The
/// integer topology engine and the reverse conversion are implementation
/// details; [`overlay`](Self::overlay) returns float curve shapes directly.
///
/// ```
/// use i_curve::{CurveBuilder, FillRule, FloatCurveOverlay, OverlayRule};
///
/// let subject = CurveBuilder::new()
///     .move_to([0.0_f64, 0.0])?
///     .line_to([10.0, 0.0])?
///     .line_to([10.0, 10.0])?
///     .close_contour()?
///     .build()?;
/// let clip = CurveBuilder::new()
///     .move_to([5.0_f64, -1.0])?
///     .line_to([12.0, -1.0])?
///     .line_to([12.0, 6.0])?
///     .close_contour()?
///     .build()?;
///
/// let result = FloatCurveOverlay::<_, i32>::new(&subject, &clip)
///     .overlay(OverlayRule::Intersect, FillRule::NonZero);
/// assert!(!result.is_empty());
/// # Ok::<(), i_curve::CurveBuildError>(())
/// ```
pub struct FloatCurveOverlay<P: FloatPointCompatible, I: CurveInt> {
    adapter: FloatPointAdapter<P, I>,
    overlay: IntCurveOverlay<I>,
}

impl<P, I> FloatCurveOverlay<P, I>
where
    P: FloatPointCompatible,
    I: CurveInt,
{
    const COORDINATE_BITS: u32 = I::BITS - CURVE_COORDINATE_SAFETY_BITS;

    /// Creates an overlay containing subject and clip curve resources.
    ///
    /// The adapter is selected from the combined bounds so both inputs use
    /// exactly the same internal grid.
    pub fn new<R0, R1>(subject: &R0, clip: &R1) -> Self
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        let bounds = combined_bounds(subject, clip);
        let adapter = FloatPointAdapter::with_coordinate_bits(bounds, Self::COORDINATE_BITS);
        Self::with_adapter(subject, Some(clip), adapter)
    }

    /// Creates an overlay containing only a subject curve resource.
    pub fn from_subject<R>(subject: &R) -> Self
    where
        R: CurveResource<P> + ?Sized,
    {
        let bounds = resource_bounds(subject).unwrap_or_else(FloatRect::zero);
        let adapter = FloatPointAdapter::with_coordinate_bits(bounds, Self::COORDINATE_BITS);
        Self::with_adapter::<R, R>(subject, None, adapter)
    }

    /// Creates an overlay with an explicit float-to-grid scale.
    ///
    /// Larger values retain smaller features but reduce the safe coordinate
    /// range. The scale is rejected when it cannot represent the combined
    /// input bounds safely.
    pub fn try_with_scale<R0, R1>(
        subject: &R0,
        clip: &R1,
        scale: P::Scalar,
    ) -> Result<Self, CurveConversionError>
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        let bounds = combined_bounds(subject, clip);
        let adapter =
            FloatPointAdapter::try_with_scale_and_coordinate_bits(bounds, scale, Self::COORDINATE_BITS)?;
        Ok(Self::with_adapter(subject, Some(clip), adapter))
    }

    fn with_adapter<R0, R1>(subject: &R0, clip: Option<&R1>, adapter: FloatPointAdapter<P, I>) -> Self
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        let capacity = resource_segment_count(subject) + clip.map_or(0, resource_segment_count);
        let mut overlay = IntCurveOverlay::with_capacity(capacity);
        add_converted_resource(&mut overlay, subject, &adapter, true);
        if let Some(clip) = clip {
            add_converted_resource(&mut overlay, clip, &adapter, false);
        }
        Self { adapter, overlay }
    }

    /// Sets the topology solver configuration.
    #[must_use]
    pub fn with_solver(mut self, solver: Solver) -> Self {
        self.overlay = self.overlay.with_solver(solver);
        self
    }

    /// Sets curve approximation options expressed in float input coordinates.
    pub fn try_with_options(
        mut self,
        options: FloatCurveOverlayOptions<P::Scalar>,
    ) -> Result<Self, FloatCurveOverlayOptionsError> {
        self.overlay = self.overlay.try_with_options(options.to_int(&self.adapter)?)?;
        Ok(self)
    }

    /// Returns the effective float-to-integer conversion scale.
    #[inline]
    pub fn scale(&self) -> P::Scalar {
        self.adapter.dir_scale()
    }

    /// Performs the Boolean operation and returns float curve shapes.
    pub fn overlay(self, overlay_rule: OverlayRule, fill_rule: FillRule) -> alloc::vec::Vec<CurveShape<P>> {
        let shapes = self.overlay.overlay(overlay_rule, fill_rule);
        convert_shapes_to_float(shapes, &self.adapter)
    }
}

fn add_converted_resource<P, I, R>(
    overlay: &mut IntCurveOverlay<I>,
    source: &R,
    adapter: &FloatPointAdapter<P, I>,
    is_subject: bool,
) where
    P: FloatPointCompatible,
    I: CurveInt,
    R: CurveResource<P> + ?Sized,
{
    let shape = convert_resource(source, adapter);
    if shape.contours.is_empty() {
        return;
    }

    let result = if is_subject {
        overlay.add_subject(shape)
    } else {
        overlay.add_clip(shape)
    };
    assert!(result.is_ok(), "float conversion produced invalid curve topology");
}

impl<P: FloatPointCompatible> CurveShape<P> {
    /// Performs a Boolean operation using the standard `i32` engine.
    pub fn overlay(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<Self> {
        FloatCurveOverlay::<P, i32>::new(self, clip).overlay(overlay_rule, fill_rule)
    }

    /// Performs a Boolean operation using an explicitly selected integer engine.
    pub fn overlay_as<I>(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<Self>
    where
        I: CurveInt,
    {
        FloatCurveOverlay::<P, I>::new(self, clip).overlay(overlay_rule, fill_rule)
    }
}

impl<P: FloatPointCompatible> CurvePath<P> {
    /// Performs a Boolean operation using the standard `i32` engine.
    pub fn overlay(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>> {
        FloatCurveOverlay::<P, i32>::new(self, clip).overlay(overlay_rule, fill_rule)
    }

    /// Performs a Boolean operation using an explicitly selected integer engine.
    pub fn overlay_as<I>(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>>
    where
        I: CurveInt,
    {
        FloatCurveOverlay::<P, I>::new(self, clip).overlay(overlay_rule, fill_rule)
    }
}

/// Convenience Boolean operations for arbitrary float curve resources.
///
/// [`CurveShape`] and [`CurvePath`] provide the same methods directly. Import
/// this trait when the subject is a slice, array, `Vec`, or another resource.
pub trait CurveResourceOverlayExt<P: FloatPointCompatible>: CurveResource<P> {
    /// Uses the default internal engine and returns float curves.
    fn overlay(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>>;

    /// Uses an explicitly selected internal engine and returns float curves.
    fn overlay_as<I>(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>>
    where
        I: CurveInt;
}

impl<P, R> CurveResourceOverlayExt<P> for R
where
    P: FloatPointCompatible,
    R: CurveResource<P> + ?Sized,
{
    #[inline]
    fn overlay(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>> {
        FloatCurveOverlay::<P, i32>::new(self, clip).overlay(overlay_rule, fill_rule)
    }

    #[inline]
    fn overlay_as<I>(
        &self,
        clip: &(impl CurveResource<P> + ?Sized),
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>>
    where
        I: CurveInt,
    {
        FloatCurveOverlay::<P, I>::new(self, clip).overlay(overlay_rule, fill_rule)
    }
}

fn resource_bounds<P, R>(resource: &R) -> Option<FloatRect<P::Scalar>>
where
    P: FloatPointCompatible,
    R: CurveResource<P> + ?Sized,
{
    resource
        .iter_paths()
        .map(CurvePath::bounds)
        .reduce(FloatRect::with_rects)
}

fn combined_bounds<P, R0, R1>(subject: &R0, clip: &R1) -> FloatRect<P::Scalar>
where
    P: FloatPointCompatible,
    R0: CurveResource<P> + ?Sized,
    R1: CurveResource<P> + ?Sized,
{
    match (resource_bounds(subject), resource_bounds(clip)) {
        (Some(subject), Some(clip)) => FloatRect::with_rects(subject, clip),
        (Some(bounds), None) | (None, Some(bounds)) => bounds,
        (None, None) => FloatRect::zero(),
    }
}

fn resource_segment_count<P, R>(resource: &R) -> usize
where
    P: FloatPointCompatible,
    R: CurveResource<P> + ?Sized,
{
    resource.iter_paths().map(|path| path.segments().len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float::arc::{Ellipse, EllipticArc};
    use crate::{CurveBuilder, FloatCurveSegment};

    fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64) -> CurveShape<[f64; 2]> {
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
    fn convenience_overlay_returns_closed_float_paths() {
        let subject = rectangle(0.0, 0.0, 10.0, 10.0);
        let clip = rectangle(5.0, 2.0, 12.0, 8.0);

        let result = subject.overlay(&clip, OverlayRule::Intersect, FillRule::NonZero);

        assert_eq!(result.len(), 1);
        let start = result[0].contours()[0].start();
        assert!((start[0] - 5.0).abs() < 1.0e-6 || (start[0] - 10.0).abs() < 1.0e-6);
    }

    #[test]
    fn rational_arcs_are_returned_in_float_coordinates() {
        let circle = EllipticArc {
            ellipse: Ellipse {
                center: [2.0_f64, 3.0],
                radius_x: 5.0,
                radius_y: 5.0,
                rotation: 0.0,
            },
            start_angle: 0.0,
            sweep_angle: core::f64::consts::TAU,
        };
        let subject = CurveBuilder::new()
            .move_to(circle.start_point())
            .unwrap()
            .arc_to(circle)
            .unwrap()
            .close_contour()
            .unwrap()
            .build()
            .unwrap();

        let result = FloatCurveOverlay::<_, i32>::from_subject(&subject)
            .overlay(OverlayRule::Subject, FillRule::NonZero);

        assert_eq!(result.len(), 1);
        assert!(
            result[0].contours()[0]
                .segments()
                .iter()
                .any(|segment| matches!(segment, FloatCurveSegment::Arc { arc } if arc.sweep_angle > 0.0))
        );
        let rebuilt = CurveShape::try_new(result[0].clone().into_contours()).unwrap();
        assert_eq!(rebuilt, result[0]);
    }

    #[test]
    fn explicit_scale_is_float_and_validated() {
        let subject = rectangle(0.0, 0.0, 10.0, 10.0);
        let clip = rectangle(5.0, 2.0, 12.0, 8.0);

        let result = FloatCurveOverlay::<_, i32>::try_with_scale(&subject, &clip, 1_000.0)
            .unwrap()
            .overlay(OverlayRule::Union, FillRule::NonZero);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn float_options_are_converted_with_the_effective_scale() {
        let subject = rectangle(0.0, 0.0, 10.0, 10.0);
        let clip = rectangle(5.0, 2.0, 12.0, 8.0);
        let options = FloatCurveOverlayOptions {
            min_chord_length: Some(0.25),
            angle_tolerance: 0.2,
            max_approximation_depth: 12,
        };

        let overlay = FloatCurveOverlay::<_, i32>::try_with_scale(&subject, &clip, 1_024.0)
            .unwrap()
            .try_with_options(options)
            .unwrap();

        assert_eq!(overlay.scale(), 1_024.0);
        assert_eq!(
            overlay.overlay.options(),
            CurveOverlayOptions {
                min_chord_length_power: 8,
                angle_tolerance_power: 3,
                max_approximation_depth: 12,
            }
        );
    }

    #[test]
    fn float_options_reject_invalid_tolerances() {
        let subject = rectangle(0.0, 0.0, 10.0, 10.0);
        let clip = rectangle(5.0, 2.0, 12.0, 8.0);

        let error = FloatCurveOverlay::<_, i32>::new(&subject, &clip)
            .try_with_options(FloatCurveOverlayOptions {
                min_chord_length: Some(0.0),
                ..Default::default()
            })
            .err();
        assert_eq!(
            error,
            Some(FloatCurveOverlayOptionsError::MinChordLengthNonPositive)
        );

        let error = FloatCurveOverlay::<_, i32>::new(&subject, &clip)
            .try_with_options(FloatCurveOverlayOptions {
                angle_tolerance: f64::NAN,
                ..Default::default()
            })
            .err();
        assert_eq!(
            error,
            Some(FloatCurveOverlayOptionsError::AngleToleranceNotFinite)
        );

        let error = FloatCurveOverlay::<_, i32>::new(&subject, &clip)
            .try_with_options(FloatCurveOverlayOptions {
                angle_tolerance: 1.1,
                ..Default::default()
            })
            .err();
        assert_eq!(
            error,
            Some(FloatCurveOverlayOptionsError::AngleToleranceOutOfRange)
        );

        let error = FloatCurveOverlay::<_, i32>::new(&subject, &clip)
            .try_with_options(
                FloatCurveOverlayOptions::default()
                    .with_max_approximation_depth(CurveOverlayOptions::MAX_APPROXIMATION_DEPTH + 1),
            )
            .err()
            .unwrap();
        assert_eq!(
            error,
            FloatCurveOverlayOptionsError::Approximation(
                CurveOverlayOptionsError::MaxApproximationDepthTooLarge {
                    requested: CurveOverlayOptions::MAX_APPROXIMATION_DEPTH + 1,
                    maximum: CurveOverlayOptions::MAX_APPROXIMATION_DEPTH,
                }
            )
        );
        assert_eq!(alloc::format!("{error}"), "invalid curve approximation options");
        let source = core::error::Error::source(&error).unwrap();
        assert!(source.is::<CurveOverlayOptionsError>());
        assert_eq!(
            alloc::format!("{source}"),
            alloc::format!(
                "maximum approximation depth {} exceeds the safety limit {}",
                CurveOverlayOptions::MAX_APPROXIMATION_DEPTH + 1,
                CurveOverlayOptions::MAX_APPROXIMATION_DEPTH
            )
        );
        assert!(source.source().is_none());
    }

    #[test]
    fn operand_collapsed_by_shared_grid_behaves_as_empty() {
        let subject = rectangle(0.0, 0.0, 10.0, 10.0);
        let clip = rectangle(5.0, 5.0, 5.0 + 1.0e-12, 5.0 + 1.0e-12);

        let intersection = subject.overlay(&clip, OverlayRule::Intersect, FillRule::NonZero);
        let union = subject.overlay(&clip, OverlayRule::Union, FillRule::NonZero);

        assert!(intersection.is_empty());
        assert_eq!(union.len(), 1);
    }
}
