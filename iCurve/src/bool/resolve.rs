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

#[cfg(test)]
mod debug_svg_tests {
    extern crate std;

    use super::*;
    use crate::curve::builder::{CurveError, CurveShapeBuilder};
    use i_overlay::core::overlay::ShapeType;
    use std::format;
    use std::fs;
    use std::println;
    use std::string::String;
    use std::vec;
    use std::vec::Vec;

    #[test]
    #[ignore]
    fn dump_union_crash_input_edges_svg() -> Result<(), CurveError> {
        let subject = vec![
            CurveShapeBuilder::new()
                .move_to([-210.0_f32, -130.0])?
                .line_to([70.0, -130.0])?
                .line_to([70.0, 130.0])?
                .line_to([-216.049_59, 129.983_02])?
                .line_to([-210.0, -130.0])?
                .build()?,
        ];
        let clip = vec![
            CurveShapeBuilder::new()
                .move_to([-70.0_f32, -170.0])?
                .line_to([210.0, -170.0])?
                .line_to([210.0, 90.0])?
                .line_to([-70.0, 90.0])?
                .line_to([-70.0, -170.0])?
                .build()?,
        ];

        let overlay = CurveOverlay::<[f32; 2], i32>::with_subj_and_clip(&subject, &clip);
        let sub_segments = overlay.split();
        let mut edges = Vec::with_capacity(sub_segments.len());

        for sub_segment in sub_segments {
            let segment = &overlay.segments[sub_segment.segment_index];
            let edge = sub_segment.edge(&segment.normalized_segment, &overlay.adapter);
            edges.push((
                segment.shape_type,
                sub_segment.segment_index,
                sub_segment.t0,
                sub_segment.t1,
                edge.a,
                edge.b,
                overlay.adapter.int_to_float(&edge.a),
                overlay.adapter.int_to_float(&edge.b),
            ));
        }

        let svg = input_edges_svg(
            overlay.adapter.dir_scale(),
            overlay.adapter.offset(),
            overlay.adapter.rect().min_x,
            overlay.adapter.rect().max_x,
            overlay.adapter.rect().min_y,
            overlay.adapter.rect().max_y,
            &edges,
        );
        let path = "/private/tmp/icurve_i_overlay_input_edges.svg";
        fs::write(path, svg).expect("svg dump must be writable");

        for (index, (shape_type, segment_index, t0, t1, a, b, fa, fb)) in edges.iter().enumerate() {
            println!(
                "{index}: {shape_type:?} segment={segment_index} t=[{t0}, {t1}] int=({}, {}) -> ({}, {}) float=({}, {}) -> ({}, {})",
                a.x, a.y, b.x, b.y, fa[0], fa[1], fb[0], fb[1]
            );
        }
        println!("wrote {path}");

        Ok(())
    }

    fn input_edges_svg(
        scale: f32,
        offset: [f32; 2],
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        edges: &[(
            ShapeType,
            usize,
            f32,
            f32,
            i_overlay::i_float::int::point::IntPoint<i32>,
            i_overlay::i_float::int::point::IntPoint<i32>,
            [f32; 2],
            [f32; 2],
        )],
    ) -> String {
        let pad = 36.0;
        let width = max_x - min_x + pad * 2.0 + 250.0;
        let height = max_y - min_y + pad * 2.0;
        let x0 = min_x - pad;
        let y0 = -max_y - pad;
        let title_y = -max_y - 18.0;

        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0} {y0} {width} {height}" width="{width}" height="{height}">
<defs>
  <marker id="arrow-subj" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto" markerUnits="strokeWidth">
    <path d="M 0 0 L 8 4 L 0 8 z" fill="#c62828"/>
  </marker>
  <marker id="arrow-clip" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto" markerUnits="strokeWidth">
    <path d="M 0 0 L 8 4 L 0 8 z" fill="#1565c0"/>
  </marker>
</defs>
<rect x="{x0}" y="{y0}" width="{width}" height="{height}" fill="#fff"/>
<text x="{min_x}" y="{title_y}" font-family="Menlo, monospace" font-size="12" fill="#111">iCurve -> iOverlay input edges, scale={scale}, offset=({:.6}, {:.6})</text>
<g font-family="Menlo, monospace" font-size="9">
"##,
            offset[0], offset[1]
        );

        for (index, (shape_type, segment_index, t0, t1, a, b, fa, fb)) in edges.iter().enumerate() {
            let is_subject = *shape_type == ShapeType::Subject;
            let color = if is_subject { "#c62828" } else { "#1565c0" };
            let marker = if is_subject { "arrow-subj" } else { "arrow-clip" };
            let prefix = if is_subject { "S" } else { "C" };
            let ax = fa[0];
            let ay = -fa[1];
            let bx = fb[0];
            let by = -fb[1];
            let mx = (ax + bx) * 0.5;
            let my = (ay + by) * 0.5;

            svg.push_str(&format!(
                r##"<line x1="{ax}" y1="{ay}" x2="{bx}" y2="{by}" stroke="{color}" stroke-width="2.5" marker-end="url(#{marker})"/>
<circle cx="{ax}" cy="{ay}" r="3" fill="{color}"/>
<text x="{:.3}" y="{:.3}" fill="{color}">{prefix}{index} seg={segment_index} [{t0:.3},{t1:.3}]</text>
<text x="{:.3}" y="{:.3}" fill="#333">({},{}) -> ({},{})</text>
"##,
                mx + 4.0,
                my - 4.0,
                mx + 4.0,
                my + 8.0,
                a.x,
                a.y,
                b.x,
                b.y
            ));
        }

        let legend_x = max_x + 55.0;
        let mut legend_y = -max_y + 5.0;
        svg.push_str(&format!(
            r##"<rect x="{}" y="{}" width="220" height="{}" fill="#fafafa" stroke="#ddd"/>
<text x="{legend_x}" y="{legend_y}" fill="#c62828">red: Subject</text>
"##,
            legend_x - 10.0,
            legend_y - 18.0,
            35 + edges.len() * 22
        ));
        legend_y += 14.0;
        svg.push_str(&format!(
            r##"<text x="{legend_x}" y="{legend_y}" fill="#1565c0">blue: Clip</text>
"##
        ));
        legend_y += 18.0;
        for (index, (shape_type, segment_index, _, _, a, b, _, _)) in edges.iter().enumerate() {
            let prefix = if *shape_type == ShapeType::Subject {
                "S"
            } else {
                "C"
            };
            let color = if *shape_type == ShapeType::Subject {
                "#c62828"
            } else {
                "#1565c0"
            };
            svg.push_str(&format!(
                r##"<text x="{legend_x}" y="{legend_y}" fill="{color}">{prefix}{index}: seg {segment_index} ({},{}) -> ({},{})</text>
"##,
                a.x, a.y, b.x, b.y
            ));
            legend_y += 14.0;
        }

        svg.push_str("</g>\n</svg>\n");
        svg
    }
}
