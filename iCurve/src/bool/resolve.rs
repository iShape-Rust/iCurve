use crate::bool::meta::{MetaSegment, ResolvedCurveOverlay};
use crate::bool::overlay::CurveOverlay;
use crate::flatten::segment::{
    NormalizedSegment, SegmentRange,
};
use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_overlay::core::edge_overlay::{EdgeOverlay, InputEdge};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_tree::{Expiration, LayoutNumber};
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::point::PointAt;

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(super) fn resolve(
        &mut self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> ResolvedCurveOverlay<I, P::Scalar>
    where
        I: Expiration + LayoutNumber + SortKey,
        P::Scalar: Send + Sync,
    {
        let mut overlay = self.edge_overlay();

        let shapes = overlay.build_vector_shapes(overlay_rule, fill_rule);
        let store = overlay.into_data_store();

        ResolvedCurveOverlay { shapes, store }
    }

    fn edge_overlay(&self) -> EdgeOverlay<I, MetaSegment<P::Scalar>>
    where
        I: Expiration + LayoutNumber + SortKey,
        P::Scalar: Send + Sync,
    {
        let ranges = self.make_ranges();
        let mut overlay = EdgeOverlay::<I, MetaSegment<P::Scalar>>::new(ranges.len());

        for range in ranges {
            let segment = &self.segments[range.segment_index];
            let edge = range.edge(&segment.normalized_segment, &self.adapter);
            overlay.add_edge(edge, segment.shape_type);
        }

        overlay
    }

    pub fn linear_edges(&self) -> Vec<[[P::Scalar; 2]; 2]>
    where
        I: Expiration + LayoutNumber + SortKey,
        P::Scalar: Send + Sync,
    {
        self.edge_overlay()
            .edges()
            .map(|[a, b]| {
                let a = self.adapter.int_to_float(&a);
                let b = self.adapter.int_to_float(&b);
                [[a.x(), a.y()], [b.x(), b.y()]]
            })
            .collect()
    }
}

impl<F: FloatNumber> SegmentRange<F> {
    fn edge<P, I>(
        self,
        segment: &NormalizedSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> InputEdge<I, MetaSegment<F>>
    where
        P: FloatPointCompatible<Scalar = F>,
        I: IntNumber,
    {
        InputEdge {
            a: adapter.float_to_int(&segment.point_at(self.t0)),
            b: adapter.float_to_int(&segment.point_at(self.t1)),
            data: MetaSegment::single(self),
        }
    }
}

trait SegmentPointAt<P: FloatPointCompatible> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> FloatPoint<P::Scalar>;
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for NormalizedSegment<P> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P {
        match self {
            Self::Line(segment) => segment.point_at(t),
            Self::Quad(segment) => segment.point_at(t),
            Self::Cubic(segment) => segment.point_at(t),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use i_overlay::i_float::float::rect::FloatRect;
    use crate::kernel::curve::line::LineSegment;

    #[test]
    fn input_edge_uses_exact_segment_endpoint_at_one() {
        let segment = NormalizedSegment::Line(LineSegment {
            control_points: [[-220.0_f32, 141.000_02].into(), [-210.0, -130.0].into()],
        });
        let adapter = FloatPointAdapter::<[f32; 2], i32>::with_scale(
            FloatRect::new(-220.0, 70.0, -130.0, 141.000_02),
            100_000.0,
        );
        let sub_segment = SegmentRange::full(0);

        let interpolated_y = 141.000_02_f32 + (-130.0_f32 - 141.000_02_f32);
        assert_ne!(interpolated_y, -130.0);

        let edge = sub_segment.edge(&segment, &adapter);

        assert_eq!(segment.point_at(SegmentParam::End), [-210.0, -130.0]);
        assert_eq!(edge.b, adapter.float_to_int(&[-210.0, -130.0]));
    }
}
