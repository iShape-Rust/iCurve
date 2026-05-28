use crate::curve::resource::CurveResource;
use crate::curve::shape::CurveShape;
use crate::flatten::convert::ShapeToSegments;
use crate::flatten::rect::ShapeFloatRect;
use crate::flatten::segment::Segment;
use alloc::vec::Vec;
use core::marker::PhantomData;
use i_key_sort::sort::key::SortKey;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_tree::{Expiration, LayoutNumber};

#[derive(Debug, Clone, Copy)]
pub struct CurveSplitOptions<F: FloatNumber, I: IntNumber = i32> {
    /// Curves with an angle above this threshold, in radians, are forcibly split in half.
    pub max_angle: F,

    /// Curves with an angle below this threshold, in radians, are not split further.
    pub min_angle: F,

    /// Minimum segment length in integer adapter coordinates.
    pub min_length: I,
}

#[derive(Debug, Clone, Copy)]
pub struct CurveReconstructOptions<F: FloatNumber> {
    phantom_data: PhantomData<F>,
}

#[derive(Debug, Clone, Copy)]
pub struct CurveOverlayOptions<F: FloatNumber, I: IntNumber = i32> {
    pub min_output_area: F,
    pub split: CurveSplitOptions<F, I>,
    pub reconstruct: CurveReconstructOptions<F>,
    phantom_data: PhantomData<I>,
}

pub struct CurveOverlay<P: FloatPointCompatible, I: IntNumber = i32> {
    pub(crate) segments: Vec<Segment<P>>,
    pub(super) adapter: FloatPointAdapter<P, I>,
    pub(crate) options: CurveOverlayOptions<P::Scalar, I>,
}

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub fn with_subj_and_clip<R0, R1>(subj: &R0, clip: &R1) -> Self
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        Self::with_subj_and_clip_custom(subj, clip, CurveOverlayOptions::default())
    }

    pub fn with_subj_and_clip_custom<R0, R1>(
        subj: &R0,
        clip: &R1,
        options: CurveOverlayOptions<P::Scalar, I>,
    ) -> Self
    where
        R0: CurveResource<P> + ?Sized,
        R1: CurveResource<P> + ?Sized,
    {
        let subj_rect = Self::resource_rect(subj);
        let clip_rect = Self::resource_rect(clip);
        let rect = FloatRect::with_optional_rects(subj_rect, clip_rect).unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::new(rect);

        let capacity = Self::resource_segments_count(subj) + Self::resource_segments_count(clip);
        let mut segments = Vec::with_capacity(capacity);

        Self::append_resource_segments(subj, ShapeType::Subject, &adapter, &mut segments);
        Self::append_resource_segments(clip, ShapeType::Clip, &adapter, &mut segments);

        Self {
            segments,
            adapter,
            options,
        }
    }

    pub fn overlay(&mut self, overlay_rule: OverlayRule, fill_rule: FillRule) -> Vec<CurveShape<P>>
    where
        I: Expiration + LayoutNumber + SortKey,
        P::Scalar: Send + Sync,
    {
        let resolved = self.resolve(overlay_rule, fill_rule);
        self.recombine(resolved)
    }

    pub(super) fn append_resource_segments<R: CurveResource<P> + ?Sized>(
        resource: &R,
        shape_type: ShapeType,
        adapter: &FloatPointAdapter<P, I>,
        output: &mut Vec<Segment<P>>,
    ) {
        for contour in resource.iter_contours() {
            output.extend(contour.to_normalize_segments_with_adapter(shape_type, adapter));
        }
    }

    pub(super) fn resource_rect<R: CurveResource<P> + ?Sized>(resource: &R) -> Option<FloatRect<P::Scalar>> {
        let mut rect = None;
        for contour in resource.iter_contours() {
            rect = FloatRect::with_optional_rects(rect, contour.float_rect());
        }
        rect
    }

    pub(super) fn resource_segments_count<R: CurveResource<P> + ?Sized>(resource: &R) -> usize {
        resource
            .iter_contours()
            .fold(0, |count, contour| count + contour.segments.len())
    }
}

impl<F: FloatNumber, I: IntNumber> Default for CurveOverlayOptions<F, I> {
    fn default() -> Self {
        Self {
            min_output_area: F::from_float(0.0),
            split: CurveSplitOptions::default(),
            reconstruct: CurveReconstructOptions::default(),
            phantom_data: Default::default(),
        }
    }
}

impl<F: FloatNumber, I: IntNumber> Default for CurveSplitOptions<F, I> {
    fn default() -> Self {
        Self {
            max_angle: F::from_float(core::f64::consts::FRAC_PI_8),
            min_angle: F::from_float(core::f64::consts::PI / 128.0),
            min_length: I::from_usize(16),
        }
    }
}

impl<F: FloatNumber> Default for CurveReconstructOptions<F> {
    fn default() -> Self {
        Self {
            phantom_data: Default::default(),
        }
    }
}
