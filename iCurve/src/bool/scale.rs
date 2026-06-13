use crate::bool::overlay::{CurveOverlay, CurveOverlayOptions};
use crate::curve::resource::CurveResource;
use crate::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::{
    FloatPointAdapter, FloatPointAdapterRangeError, FloatPointAdapterScaleError,
};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_tree::{Expiration, LayoutNumber};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedScaleCurveOverlayError {
    /// Requested scale is larger than the safe adapter scale for the input bounds.
    ScaleTooLarge,
    /// Requested scale is zero or negative.
    ScaleNonPositive,
    /// Requested scale is NaN or infinite.
    ScaleNotFinite,
    /// A curve point is outside the adapter rectangle.
    PointOutOfRange,
}

impl From<FloatPointAdapterScaleError> for FixedScaleCurveOverlayError {
    #[inline]
    fn from(error: FloatPointAdapterScaleError) -> Self {
        match error {
            FloatPointAdapterScaleError::ScaleTooLarge => Self::ScaleTooLarge,
            FloatPointAdapterScaleError::ScaleNonPositive => Self::ScaleNonPositive,
            FloatPointAdapterScaleError::ScaleNotFinite => Self::ScaleNotFinite,
        }
    }
}

impl From<FloatPointAdapterRangeError> for FixedScaleCurveOverlayError {
    #[inline]
    fn from(error: FloatPointAdapterRangeError) -> Self {
        match error {
            FloatPointAdapterRangeError::PointOutOfRange => Self::PointOutOfRange,
        }
    }
}

/// Trait for curve boolean operations with a fixed float-to-integer scale.
///
/// The `scale` parameter defines the float-to-integer conversion used by the
/// integer overlay engine:
/// `x_int = (x_float - offset_x) * scale`.
///
pub trait FixedScaleCurveOverlay<R1, P>
where
    R1: CurveResource<P> + ?Sized,
    P: FloatPointCompatible,
    P::Scalar: Send + Sync,
{
    /// Runs a curve boolean operation with the default integer engine (`i32`).
    fn overlay_with_fixed_scale(
        &self,
        clip: &R1,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
        scale: P::Scalar,
    ) -> Result<Vec<CurveShape<P>>, FixedScaleCurveOverlayError>;

    /// Same as [`Self::overlay_with_fixed_scale`], but with an explicit integer engine.
    fn overlay_with_fixed_scale_as<I>(
        &self,
        clip: &R1,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
        scale: P::Scalar,
    ) -> Result<Vec<CurveShape<P>>, FixedScaleCurveOverlayError>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey;
}

impl<R0, R1, P> FixedScaleCurveOverlay<R1, P> for R0
where
    R0: CurveResource<P> + ?Sized,
    R1: CurveResource<P> + ?Sized,
    P: FloatPointCompatible,
    P::Scalar: Send + Sync,
{
    #[inline]
    fn overlay_with_fixed_scale(
        &self,
        clip: &R1,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
        scale: P::Scalar,
    ) -> Result<Vec<CurveShape<P>>, FixedScaleCurveOverlayError> {
        let mut overlay = CurveOverlay::<P>::with_subj_and_clip_fixed_scale(self, clip, scale)?;
        Ok(overlay.overlay(overlay_rule, fill_rule))
    }

    #[inline]
    fn overlay_with_fixed_scale_as<I>(
        &self,
        clip: &R1,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
        scale: P::Scalar,
    ) -> Result<Vec<CurveShape<P>>, FixedScaleCurveOverlayError>
    where
        I: IntNumber + Expiration + LayoutNumber + SortKey,
    {
        let mut overlay = CurveOverlay::<P, I>::with_subj_and_clip_fixed_scale(self, clip, scale)?;
        Ok(overlay.overlay(overlay_rule, fill_rule))
    }
}

impl<P, I> CurveOverlay<P, I>
where
    P: FloatPointCompatible,
    I: IntNumber,
{
    /// Creates a curve overlay using a fixed float-to-integer scale.
    ///
    /// The requested scale is validated against the combined input bounds.
    pub fn with_subj_and_clip_fixed_scale<R0, R1>(
        subj: &R0,
        clip: &R1,
        scale: P::Scalar,
    ) -> Result<Self, FixedScaleCurveOverlayError>
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        Self::from_subj_and_clip_fixed_scale(subj, clip, scale)
    }

    /// Creates a curve overlay using a fixed float-to-integer scale.
    ///
    /// This is the explicit-engine constructor; select the integer engine on
    /// `CurveOverlay<P, I>`.
    pub fn from_subj_and_clip_fixed_scale<R0, R1>(
        subj: &R0,
        clip: &R1,
        scale: P::Scalar,
    ) -> Result<Self, FixedScaleCurveOverlayError>
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        Self::from_subj_and_clip_fixed_scale_custom(subj, clip, CurveOverlayOptions::default(), scale)
    }

    /// Creates a curve overlay using custom options and a fixed float-to-integer scale.
    ///
    /// The requested scale is validated against the combined input bounds.
    pub fn with_subj_and_clip_fixed_scale_custom<R0, R1>(
        subj: &R0,
        clip: &R1,
        options: CurveOverlayOptions<P::Scalar, I>,
        scale: P::Scalar,
    ) -> Result<Self, FixedScaleCurveOverlayError>
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        Self::from_subj_and_clip_fixed_scale_custom(subj, clip, options, scale)
    }

    /// Creates a curve overlay using custom options and a fixed float-to-integer scale.
    ///
    /// This is the explicit-engine constructor; select the integer engine on
    /// `CurveOverlay<P, I>`.
    pub fn from_subj_and_clip_fixed_scale_custom<R0, R1>(
        subj: &R0,
        clip: &R1,
        options: CurveOverlayOptions<P::Scalar, I>,
        scale: P::Scalar,
    ) -> Result<Self, FixedScaleCurveOverlayError>
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        let subj_rect = Self::resource_rect(subj);
        let clip_rect = Self::resource_rect(clip);
        let rect = FloatRect::with_optional_rects(subj_rect, clip_rect).unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::try_with_scale(rect, scale)?;

        let capacity = Self::resource_segments_count(subj) + Self::resource_segments_count(clip);
        let mut segments = Vec::with_capacity(capacity);
        let point_adapter = adapter.to_float_point_adapter();

        Self::append_resource_segments(subj, ShapeType::Subject, &point_adapter, &mut segments)?;
        Self::append_resource_segments(clip, ShapeType::Clip, &point_adapter, &mut segments)?;

        Ok(Self {
            segments,
            adapter,
            options,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::builder::CurveBuilder;

    #[test]
    fn fixed_scale_constructor_uses_requested_scale() {
        let subj = square(0.0, 0.0, 10.0);
        let clip = square(2.0, 2.0, 2.0);

        let overlay = CurveOverlay::<[f64; 2], i32>::with_subj_and_clip_fixed_scale(&subj, &clip, 1.0)
            .expect("scale must be valid for small test coordinates");

        assert_eq!(overlay.adapter.dir_scale(), 1.0);
    }

    #[test]
    fn fixed_scale_trait_runs_overlay() {
        let subj = square(0.0, 0.0, 10.0);
        let clip = square(2.0, 2.0, 2.0);

        let result = subj
            .overlay_with_fixed_scale(&clip, OverlayRule::Difference, FillRule::EvenOdd, 1.0)
            .expect("scale must be valid for small test coordinates");

        assert!(!result.is_empty());
    }

    #[test]
    fn fixed_scale_rejects_non_positive_scale() {
        let subj = square(0.0, 0.0, 10.0);
        let clip = square(2.0, 2.0, 2.0);

        let error = match CurveOverlay::<[f64; 2], i32>::with_subj_and_clip_fixed_scale(&subj, &clip, 0.0) {
            Ok(_) => panic!("zero scale must be rejected"),
            Err(error) => error,
        };

        assert_eq!(error, FixedScaleCurveOverlayError::ScaleNonPositive);
    }

    fn square(x: f64, y: f64, size: f64) -> CurveShape<[f64; 2]> {
        CurveBuilder::new()
            .move_to([x, y])
            .unwrap()
            .line_to([x + size, y])
            .unwrap()
            .line_to([x + size, y + size])
            .unwrap()
            .line_to([x, y + size])
            .unwrap()
            .line_to([x, y])
            .unwrap()
            .build_shape()
            .unwrap()
    }
}
