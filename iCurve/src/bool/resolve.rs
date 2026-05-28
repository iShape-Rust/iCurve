use crate::bool::overlay::CurveOverlay;
use crate::flatten::segment::{
    ArcSegment, CubicSegment, LineSegment, NormalizedSegment, QuadSegment, SubSegment,
};
use crate::flatten::split::SplitAt;
use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_overlay::core::edge_data::{EdgeDataMerge, EdgeDataSplit, OverlayEdgeData};
use i_overlay::core::edge_overlay::{EdgeOverlay, InputEdge};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::segm::boolean::ShapeCountBoolean;
use i_overlay::vector::edge::DataVectorShape;
use i_tree::{Expiration, LayoutNumber};

impl<P: FloatPointCompatible, I: IntNumber> CurveOverlay<P, I> {
    pub(super) fn resolve(
        &mut self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> Vec<DataVectorShape<I, SubSegment<P::Scalar>>>
    where
        I: Expiration + LayoutNumber + SortKey,
        P::Scalar: Send + Sync,
    {
        let sub_segments = self.split();
        let mut overlay = EdgeOverlay::<I, SubSegment<P::Scalar>>::new(sub_segments.len());

        for sub_segment in sub_segments {
            let segment = &self.segments[sub_segment.segment_index];
            let edge = sub_segment.edge(&segment.normalized_segment, &self.adapter);
            overlay.add_edge(edge, segment.shape_type);
        }

        overlay.build_vector_shapes(overlay_rule, fill_rule)
    }
}

impl<F: FloatNumber + Send + Sync> OverlayEdgeData for SubSegment<F> {
    fn merge(ctx: EdgeDataMerge<ShapeCountBoolean, Self>) -> Self {
        if ctx.out_count == ctx.lhs_count {
            ctx.lhs_data
        } else {
            ctx.rhs_data
        }
    }

    #[inline(always)]
    fn reversed(self) -> Self {
        Self {
            segment_index: self.segment_index,
            t0: self.t1,
            t1: self.t0,
        }
    }

    #[inline(always)]
    fn split<I: IntNumber>(self, ctx: EdgeDataSplit<I>) -> (Self, Self) {
        let ratio = split_ratio(ctx);
        let tm = self.t0 + (self.t1 - self.t0) * F::from_float(ratio);
        (
            Self {
                segment_index: self.segment_index,
                t0: self.t0,
                t1: tm,
            },
            Self {
                segment_index: self.segment_index,
                t0: tm,
                t1: self.t1,
            },
        )
    }
}

impl<F: FloatNumber> SubSegment<F> {
    fn edge<P, I>(
        self,
        segment: &NormalizedSegment<P>,
        adapter: &FloatPointAdapter<P, I>,
    ) -> InputEdge<I, Self>
    where
        P: FloatPointCompatible<Scalar = F>,
        I: IntNumber,
    {
        InputEdge {
            a: adapter.float_to_int(&segment.point_at(self.t0)),
            b: adapter.float_to_int(&segment.point_at(self.t1)),
            data: self,
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
        let [left, _] = self.split_at(t);
        left.control_points[1]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for QuadSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        let [left, _] = self.split_at(t);
        left.control_points[2]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for CubicSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
        let [left, _] = self.split_at(t);
        left.control_points[3]
    }
}

impl<P: FloatPointCompatible> SegmentPointAt<P> for ArcSegment<P> {
    fn point_at(&self, t: P::Scalar) -> P {
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

fn split_ratio<I: IntNumber>(ctx: EdgeDataSplit<I>) -> f64 {
    let dx = ctx.b.x.wide() - ctx.a.x.wide();
    let dy = ctx.b.y.wide() - ctx.a.y.wide();

    let (num, den) = if dx.unsigned_abs() >= dy.unsigned_abs() {
        (ctx.p.x.wide() - ctx.a.x.wide(), dx)
    } else {
        (ctx.p.y.wide() - ctx.a.y.wide(), dy)
    };

    if den == I::Wide::ZERO {
        return 0.5;
    }

    (num.to_f64() / den.to_f64()).clamp(0.0, 1.0)
}
