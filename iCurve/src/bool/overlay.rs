use alloc::vec::Vec;
use core::marker::PhantomData;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::{ShapeType};
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use crate::curve::shape::CurveShape;
use crate::flatten::convert::ShapeToSegments;
use crate::flatten::rect::ShapeFloatRect;
use crate::flatten::segment::{Segment};

pub struct CurveOverlayOptions<F: FloatNumber, I: IntNumber> {
    pub min_output_area: F,
    phantom_data: PhantomData<I>,
}

pub struct CurveOverlay<P: FloatPointCompatible, I: IntNumber> {
    pub(crate) segments: Vec<Segment<P>>,
    pub(super) adapter: FloatPointAdapter<P, I>,
    pub(crate) options: CurveOverlayOptions<P::Scalar, I>,
}

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub fn with_shapes_custom(
        subj: &[CurveShape<P>],
        clip: &[CurveShape<P>],
        options: CurveOverlayOptions<P::Scalar, I>,
    ) -> Self {
        let subj_rect = subj.float_rect();
        let clip_rect = clip.float_rect();
        let rect = FloatRect::with_optional_rects(subj_rect, clip_rect).unwrap_or(FloatRect::zero());
        let adapter = FloatPointAdapter::new(rect);

        let mut segments = Vec::new();

        for shape in subj {
            let subj_segments = shape.to_normalize_segments_with_adapter(ShapeType::Subject, &adapter);
            segments = subj_segments;
        }

        for shape in clip {
            let clip_segments = shape.to_normalize_segments_with_adapter(ShapeType::Subject, &adapter);
            segments.extend(clip_segments);
        }

        Self {
            segments,
            adapter,
            options,
        }
    }

    #[inline]
    pub fn overlay(&mut self, overlay_rule: OverlayRule, fill_rule: FillRule) -> Vec<CurveShape<P>> {
        Vec::new()
    }

}