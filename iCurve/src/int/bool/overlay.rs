use crate::int::bool::segment::ShapeSegment;
use alloc::vec::Vec;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::int::number::int::IntNumber;
use crate::int::curve::shape::CurveShape;

pub struct IntCurveOverlay<I: IntNumber> {
    pub(crate) shape_segments: Vec<ShapeSegment<I>>,
}

impl<I: IntNumber> IntCurveOverlay<I> {

    pub fn add_shape(&mut self, shape: CurveShape<I>, shape_type: ShapeType) {
        for curve in shape.contours.into_iter() {
            for segment in curve.segments.into_iter() {
                self.shape_segments.push(ShapeSegment { segment, shape_type });
            }
        }
    }

    #[inline]
    pub fn overlay(&mut self, overlay_rule: OverlayRule, fill_rule: FillRule) -> Vec<CurveShape<I>> {
        // we will have several steps
        // 1. prepare topology, split into segments and bake curve type in each segment
        // 2. use this topology find overlay using i_overlay::core::edge_overlay::EdgeOverlay
        // 3. build curve back using baked information in segments

        // Now all steps more detailed

        // 1. prepare topology
        // convert segments into kernel::int::curve::Segment but this time keep it's shape type
        // need a new struct and a good name for it
        // we also convert all segments in canonical form using PushCanonicalSegment but this time we also should keep shape type

        // find all intersection and split segments in all intersection points, the goal the result segments must not intersect each other but can touch each other at ends
        // the segments (convex hull) can not contain chord of other segments and if it has we also must break this segment. We will use special approximate api for it like find closest point to a chord to split a segment

        // 2. all prepared segments can now successfully use EdgeOverlay as a segment data we will use id/or index to shape id/index

        // 3. by EdgeOverlay result segments collect curve slices if neighbor segments has same id it's a part of the same curve slice we join it

        Vec::new()
    }


}