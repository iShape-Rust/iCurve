use crate::bool::meta::{MetaSegment, ResolvedCurveOverlay};
use crate::bool::overlay::CurveOverlay;
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, SegmentParam, SegmentRange,
};
use crate::flatten::split::SplitAt;
use i_key_sort::sort::key::SortKey;
use i_overlay::core::edge_overlay::{EdgeOverlay, InputEdge};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
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
        let ranges = self.split();
        let mut overlay = EdgeOverlay::<I, MetaSegment<P::Scalar>>::new(ranges.len());

        for range in ranges {
            let segment = &self.segments[range.segment_index];
            let edge = range.edge(&segment.normalized_segment, &self.adapter);
            overlay.add_edge(edge, segment.shape_type);
        }

        let shapes = overlay.build_vector_shapes(overlay_rule, fill_rule);
        let store = overlay.into_data_store();

        ResolvedCurveOverlay { shapes, store }
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
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P;
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for NormalizedSegment<P> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P {
        match self {
            Self::Line(segment) => segment.point_at(t),
            Self::Quad(segment) => segment.point_at(t),
            Self::Cubic(segment) => segment.point_at(t),
            Self::Arc(segment) => segment.point_at(t),
        }
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for LineSegment<P> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P {
        if t == SegmentParam::Start {
            return self.control_points[0];
        }
        if t == SegmentParam::End {
            return self.control_points[1];
        }

        let [left, _] = self.split_at(t.value());
        left.control_points[1]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for QuadSegment<P> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P {
        if t == SegmentParam::Start {
            return self.control_points[0];
        }
        if t == SegmentParam::End {
            return self.control_points[2];
        }

        let [left, _] = self.split_at(t.value());
        left.control_points[2]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for CubicSegment<P> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P {
        if t == SegmentParam::Start {
            return self.control_points[0];
        }
        if t == SegmentParam::End {
            return self.control_points[3];
        }

        let [left, _] = self.split_at(t.value());
        left.control_points[3]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for ArcSegment<P> {
    fn point_at(&self, t: SegmentParam<P::Scalar>) -> P {
        if t == SegmentParam::Start {
            return self.p0;
        }
        if t == SegmentParam::End {
            return self.p1;
        }

        let t = t.value();
        let angle = self.start_angle + self.sweep_angle * t;
        let x = self.radii.x() * angle.cos();
        let y = self.radii.y() * angle.sin();
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        P::from_xy(
            self.center.x() + x * cos - y * sin,
            self.center.y() + x * sin + y * cos,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use i_overlay::i_float::float::rect::FloatRect;

    #[test]
    fn input_edge_uses_exact_segment_endpoint_at_one() {
        let segment = NormalizedSegment::Line(LineSegment {
            control_points: [[-220.0_f32, 141.000_02], [-210.0, -130.0]],
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
