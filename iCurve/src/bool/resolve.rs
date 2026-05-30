use crate::bool::meta::{MetaSegment, ResolvedCurveOverlay};
use crate::bool::overlay::CurveOverlay;
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, SubSegment,
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
        let sub_segments = self.split();
        let mut overlay = EdgeOverlay::<I, MetaSegment<P::Scalar>>::new(sub_segments.len());

        for sub_segment in sub_segments {
            let segment = &self.segments[sub_segment.segment_index];
            let edge = sub_segment.edge(&segment.normalized_segment, &self.adapter);
            overlay.add_edge(edge, segment.shape_type);
        }

        let shapes = overlay.build_vector_shapes(overlay_rule, fill_rule);
        let store = overlay.into_data_store();

        ResolvedCurveOverlay { shapes, store }
    }
}

impl<F: FloatNumber> SubSegment<F> {
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
    fn point_at(&self, t: P::Scalar) -> P;
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for NormalizedSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        match self {
            Self::Line(segment) => segment.point_at(t),
            Self::Quad(segment) => segment.point_at(t),
            Self::Cubic(segment) => segment.point_at(t),
            Self::Arc(segment) => segment.point_at(t),
        }
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for LineSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        if t == P::Scalar::from_float(0.0) {
            return self.control_points[0];
        }
        if t == P::Scalar::from_float(1.0) {
            return self.control_points[1];
        }

        let [left, _] = self.split_at(t);
        left.control_points[1]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for QuadSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        if t == P::Scalar::from_float(0.0) {
            return self.control_points[0];
        }
        if t == P::Scalar::from_float(1.0) {
            return self.control_points[2];
        }

        let [left, _] = self.split_at(t);
        left.control_points[2]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for CubicSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        if t == P::Scalar::from_float(0.0) {
            return self.control_points[0];
        }
        if t == P::Scalar::from_float(1.0) {
            return self.control_points[3];
        }

        let [left, _] = self.split_at(t);
        left.control_points[3]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for ArcSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        if t == P::Scalar::from_float(0.0) {
            return self.p0;
        }
        if t == P::Scalar::from_float(1.0) {
            return self.p1;
        }

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