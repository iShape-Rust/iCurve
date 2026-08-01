use crate::float::curve::converter::{convert_shape, convert_shapes_to_float};
use crate::float::curve::shape::CurveShape;
use crate::int::CURVE_COORDINATE_SAFETY_BITS;
use crate::int::IntCurveOverlay;
use crate::{CurveConversionError, FillRule, OverlayRule, Solver};
use i_key_sort::sort::key::SortKey;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
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
/// let result = FloatCurveOverlay::with_subj_and_clip(&subject, &clip)
///     .overlay(OverlayRule::Intersect, FillRule::NonZero);
/// assert!(!result.is_empty());
/// # Ok::<(), i_curve::CurveBuildError>(())
/// ```
pub struct FloatCurveOverlay<P: FloatPointCompatible, I: IntNumber + Expiration = i32> {
    adapter: FloatPointAdapter<P, I>,
    overlay: IntCurveOverlay<I>,
}

impl<P, I> FloatCurveOverlay<P, I>
where
    P: FloatPointCompatible,
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    const COORDINATE_BITS: u32 = I::BITS - CURVE_COORDINATE_SAFETY_BITS;

    /// Creates an overlay containing one subject and one clip shape.
    ///
    /// The adapter is selected from the combined bounds so both inputs use
    /// exactly the same internal grid.
    pub fn from_subj_and_clip(subject: &CurveShape<P>, clip: &CurveShape<P>) -> Self {
        let bounds = i_overlay::i_float::float::rect::FloatRect::with_rects(subject.bounds(), clip.bounds());
        let adapter = FloatPointAdapter::with_coordinate_bits(bounds, Self::COORDINATE_BITS);
        Self::with_adapter(subject, Some(clip), adapter)
    }

    /// Creates an overlay containing only a subject shape.
    pub fn from_subj(subject: &CurveShape<P>) -> Self {
        let adapter = FloatPointAdapter::with_coordinate_bits(subject.bounds(), Self::COORDINATE_BITS);
        Self::with_adapter(subject, None, adapter)
    }

    /// Creates an overlay with an explicit float-to-grid scale.
    ///
    /// Larger values retain smaller features but reduce the safe coordinate
    /// range. The scale is rejected when it cannot represent the combined
    /// input bounds safely.
    pub fn try_from_subj_and_clip_with_scale(
        subject: &CurveShape<P>,
        clip: &CurveShape<P>,
        scale: P::Scalar,
    ) -> Result<Self, CurveConversionError> {
        let bounds = i_overlay::i_float::float::rect::FloatRect::with_rects(subject.bounds(), clip.bounds());
        let adapter =
            FloatPointAdapter::try_with_scale_and_coordinate_bits(bounds, scale, Self::COORDINATE_BITS)?;
        Ok(Self::with_adapter(subject, Some(clip), adapter))
    }

    fn with_adapter(
        subject: &CurveShape<P>,
        clip: Option<&CurveShape<P>>,
        adapter: FloatPointAdapter<P, I>,
    ) -> Self {
        let capacity = subject.segment_count() + clip.map_or(0, CurveShape::segment_count);
        let mut overlay = IntCurveOverlay::with_capacity(capacity);
        add_converted_shape(&mut overlay, subject, &adapter, true);
        if let Some(clip) = clip {
            add_converted_shape(&mut overlay, clip, &adapter, false);
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

impl<P: FloatPointCompatible> FloatCurveOverlay<P> {
    /// Creates an overlay with automatic scaling.
    ///
    /// This spelling uses the default internal engine.
    #[inline]
    pub fn with_subj_and_clip(subject: &CurveShape<P>, clip: &CurveShape<P>) -> Self {
        Self::from_subj_and_clip(subject, clip)
    }

    /// Creates a subject-only overlay with automatic scaling.
    #[inline]
    pub fn with_subj(subject: &CurveShape<P>) -> Self {
        Self::from_subj(subject)
    }

    /// Creates an overlay with an explicit float scale.
    ///
    /// This spelling keeps the ordinary API independent of the internal
    /// integer engine while still allowing a reproducible grid resolution.
    #[inline]
    pub fn try_with_subj_and_clip_scale(
        subject: &CurveShape<P>,
        clip: &CurveShape<P>,
        scale: P::Scalar,
    ) -> Result<Self, CurveConversionError> {
        Self::try_from_subj_and_clip_with_scale(subject, clip, scale)
    }
}

fn add_converted_shape<P, I>(
    overlay: &mut IntCurveOverlay<I>,
    source: &CurveShape<P>,
    adapter: &FloatPointAdapter<P, I>,
    is_subject: bool,
) where
    P: FloatPointCompatible,
    I: IntNumber + Expiration + LayoutNumber + SortKey,
{
    let shape = convert_shape(source.clone(), adapter);
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

/// Convenience Boolean operations for a pair of float curve shapes.
pub trait SingleFloatCurveOverlay<P: FloatPointCompatible> {
    /// Uses the default internal engine and returns float curves.
    fn overlay(
        &self,
        clip: &Self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>>;

    /// Uses an explicitly selected internal engine and returns float curves.
    fn overlay_as<I>(
        &self,
        clip: &Self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey;
}

impl<P: FloatPointCompatible> SingleFloatCurveOverlay<P> for CurveShape<P> {
    #[inline]
    fn overlay(
        &self,
        clip: &Self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>> {
        FloatCurveOverlay::with_subj_and_clip(self, clip).overlay(overlay_rule, fill_rule)
    }

    #[inline]
    fn overlay_as<I>(
        &self,
        clip: &Self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> alloc::vec::Vec<CurveShape<P>>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        FloatCurveOverlay::<P, I>::from_subj_and_clip(self, clip).overlay(overlay_rule, fill_rule)
    }
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

        let result = FloatCurveOverlay::with_subj(&subject).overlay(OverlayRule::Subject, FillRule::NonZero);

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

        let result = FloatCurveOverlay::try_with_subj_and_clip_scale(&subject, &clip, 1_000.0)
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
