use crate::float::curve::converter::{convert_resource, convert_shapes_to_float};
use crate::float::curve::path::CurvePath;
use crate::float::curve::shape::CurveShape;
use crate::float::resource::CurveResource;
use crate::int::CURVE_COORDINATE_SAFETY_BITS;
use crate::int::IntCurveOverlay;
use crate::{CurveConversionError, FillRule, OverlayRule, Solver};
use i_key_sort::sort::key::SortKey;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_tree::{Expiration, LayoutNumber};

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
pub struct FloatCurveOverlay<P: FloatPointCompatible, I: IntNumber + Expiration> {
    adapter: FloatPointAdapter<P, I>,
    overlay: IntCurveOverlay<I>,
}

impl<P, I> FloatCurveOverlay<P, I>
where
    P: FloatPointCompatible,
    I: IntNumber + Expiration + LayoutNumber + SortKey,
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
    I: IntNumber + Expiration + LayoutNumber + SortKey,
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
        I: IntNumber + Expiration + LayoutNumber + SortKey,
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
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        FloatCurveOverlay::<P, I>::new(self, clip).overlay(overlay_rule, fill_rule)
    }
}

/// Convenience Boolean operations for arbitrary float curve resources.
///
/// [`CurveShape`] and [`CurvePath`] provide the same methods directly. Import
/// this trait when the subject is a slice, array, `Vec`, or another resource.
pub trait SingleFloatCurveOverlay<P: FloatPointCompatible>: CurveResource<P> {
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
        I: IntNumber + Expiration + LayoutNumber + SortKey;
}

impl<P, R> SingleFloatCurveOverlay<P> for R
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
        I: IntNumber + Expiration + LayoutNumber + SortKey,
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
        assert!(result[0].contours().iter().all(|path| path.is_closed()));
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
        assert!(result[0].contours()[0].is_closed());
        assert!(
            result[0].contours()[0]
                .segments()
                .iter()
                .any(|segment| matches!(segment, FloatCurveSegment::Arc { arc } if arc.sweep_angle > 0.0))
        );
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
    fn operand_collapsed_by_shared_grid_behaves_as_empty() {
        let subject = rectangle(0.0, 0.0, 10.0, 10.0);
        let clip = rectangle(5.0, 5.0, 5.0 + 1.0e-12, 5.0 + 1.0e-12);

        let intersection = subject.overlay(&clip, OverlayRule::Intersect, FillRule::NonZero);
        let union = subject.overlay(&clip, OverlayRule::Union, FillRule::NonZero);

        assert!(intersection.is_empty());
        assert_eq!(union.len(), 1);
        assert!(union[0].contours()[0].is_closed());
    }
}
