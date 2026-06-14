use crate::bool::meta::{MetaSegment, ResolvedCurveOverlay};
use crate::bool::overlay::CurveOverlay;
use crate::bool::segment::SegmentRange;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::point_at::PointAt;
use crate::kernel::curve::segment::Segment;
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
        let point_adapter = self.adapter.to_float_point_adapter();

        for range in ranges {
            let segment = &self.segments[range.segment_index];
            let edge = range.edge(&segment.segment, &point_adapter);
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

impl<T: FloatNumber> SegmentRange<T> {
    fn edge<I>(
        self,
        segment: &Segment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> InputEdge<I, MetaSegment<T>>
    where
        I: IntNumber,
    {
        InputEdge {
            a: adapter.float_to_int(&segment.point_at(self.t0)),
            b: adapter.float_to_int(&segment.point_at(self.t1)),
            data: MetaSegment::single(self),
        }
    }
}

trait SegmentPointAt<T: FloatNumber> {
    fn point_at(&self, t: SegmentParam<T>) -> FloatPoint<T>;
}

impl<T: FloatNumber> SegmentPointAt<T> for Segment<T> {
    fn point_at(&self, t: SegmentParam<T>) -> FloatPoint<T> {
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
    use crate::kernel::curve::line::LineSegment;
    use i_overlay::i_float::float::rect::FloatRect;

    #[test]
    fn input_edge_uses_exact_segment_endpoint_at_one() {
        let segment = Segment::Line(LineSegment {
            control_points: [[-220.0_f32, 141.000_02].into(), [-210.0, -130.0].into()],
        });
        let adapter = FloatPointAdapter::<_, i32>::with_scale(
            FloatRect::new(-220.0, 70.0, -130.0, 141.000_02),
            100_000.0,
        );
        let sub_segment = SegmentRange::full(0);

        let interpolated_y = 141.000_02_f32 + (-130.0_f32 - 141.000_02_f32);
        assert_ne!(interpolated_y, -130.0);

        let edge = sub_segment.edge(&segment, &adapter);

        let expected = FloatPoint::new(-210.0, -130.0);
        let actual = segment.point_at(SegmentParam::End);
        assert_eq!(actual.x, expected.x);
        assert_eq!(actual.y, expected.y);
        assert_eq!(edge.b, adapter.float_to_int(&expected));
    }
}
