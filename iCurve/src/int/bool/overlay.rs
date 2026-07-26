use crate::int::bool::chord_refine::{ChordTopologyRefiner, RefineOutcome};
use crate::int::bool::data::{CurveEdgeData, CurveEdgeDataStore};
use crate::int::bool::edge::CurveEdge;
use crate::int::bool::planarize::CurvePlanarizer;
use crate::int::bool::recompose::CurveRecomposer;
use crate::int::bool::slice::{CurveId, CurveMark, CurveSlice};
use crate::int::curve::shape::CurveShape;
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::normalization::canonical::{PushCanonicalSimpleParametricSegment, PushSimpleSegment};
use alloc::vec::Vec;
use i_key_sort::sort::key::SortKey;
use i_overlay::core::edge_overlay::{EdgeOverlay, InputEdge};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::vector::edge::DataVectorShape;
use i_tree::{Expiration, LayoutNumber};

pub struct IntCurveOverlay<I: IntNumber> {
    pub(crate) curve_slices: Vec<CurveSlice<I>>,
    pub(crate) curve_edges: Vec<CurveEdge<I>>,
}

impl<I: IntNumber> IntCurveOverlay<I> {
    pub fn new(capacity: usize) -> Self {
        Self {
            curve_slices: Vec::with_capacity(capacity),
            curve_edges: Vec::with_capacity(capacity),
        }
    }

    pub fn add_shape(&mut self, shape: CurveShape<I>, shape_type: ShapeType) {
        let mut simple_curves = Vec::new();
        let mut canonical_curves = Vec::new();

        for contour in shape.contours {
            let mut current = contour.start;

            for segment in contour.segments {
                let (curve, end) = segment.into_kernel_segment(current);
                simple_curves.clear();
                simple_curves.push_simple(curve);

                for simple_curve in simple_curves.drain(..) {
                    let curve_id = CurveId(self.curve_slices.len());
                    canonical_curves.clear();
                    canonical_curves.push_canonical_simple_parametric(simple_curve);

                    let mut curve_slice = CurveSlice::new(simple_curve, shape_type);
                    for canonical in &canonical_curves {
                        let chord = canonical.curve.chord();
                        curve_slice.add_mark(CurveMark {
                            point: chord.a,
                            param: canonical.start,
                        });
                        curve_slice.add_mark(CurveMark {
                            point: chord.b,
                            param: canonical.end,
                        });
                    }
                    self.curve_slices.push(curve_slice);

                    self.curve_edges
                        .extend(canonical_curves.drain(..).map(|canonical| {
                            CurveEdge::new(canonical.curve, curve_id, canonical.start, canonical.end)
                        }));
                }

                current = end;
            }
        }
    }

    fn prepare(&mut self) {
        // Split curves until their chords preserve the planar curve topology.
        let mut planarizer = CurvePlanarizer::new();
        let mut chord_refiner = ChordTopologyRefiner::new();

        loop {
            planarizer.planarize(&mut self.curve_edges, &mut self.curve_slices);

            match chord_refiner.refine(&mut self.curve_edges, &mut self.curve_slices) {
                RefineOutcome::PlanarityPreserved => break,
                RefineOutcome::Replanarize {
                    escaped_marks,
                    crossed_chords,
                } => {
                    #[cfg(all(debug_assertions, feature = "std"))]
                    std::eprintln!(
                        "ChordTopologyRefiner: {escaped_marks} marks escaped their source hull and {crossed_chords} chord crossings were refined; running CurvePlanarizer again"
                    );

                    #[cfg(not(all(debug_assertions, feature = "std")))]
                    let _ = (escaped_marks, crossed_chords);
                }
            }
        }
    }

    fn build_vector_shapes(
        &self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> (Vec<DataVectorShape<I, CurveEdgeData>>, CurveEdgeDataStore)
    where
        I: Expiration + LayoutNumber + SortKey,
    {
        let mut edge_overlay = EdgeOverlay::new(self.curve_edges.len());

        for edge in &self.curve_edges {
            let chord = edge.curve.chord();
            let curve_slice = &self.curve_slices[edge.curve_id.0];
            edge_overlay.add_edge(
                InputEdge {
                    a: chord.a,
                    b: chord.b,
                    data: CurveEdgeData::Single(edge.curve_id),
                },
                curve_slice.shape_type,
            );
        }

        let shapes = edge_overlay.build_vector_shapes(overlay_rule, fill_rule);
        let data_store = edge_overlay.into_data_store();

        (shapes, data_store)
    }

    #[inline]
    pub fn overlay(&mut self, overlay_rule: OverlayRule, fill_rule: FillRule) -> Vec<CurveShape<I>>
    where
        I: Expiration + LayoutNumber + SortKey,
    {
        self.prepare();

        // Resolve the boolean topology while preserving CurveId provenance.
        let (vector_shapes, data_store) = self.build_vector_shapes(overlay_rule, fill_rule);

        // Restore maximal runs from their source curve slices.
        CurveRecomposer::new().recompose(vector_shapes, &data_store, &self.curve_slices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::curve::path::CurvePath;
    use crate::int::curve::segment::CurveSegment;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcSegment, ArcVector, EllipseFrame};
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;
    use i_overlay::i_shape::int::IntPoint;

    fn circle(center: IntPoint<i32>) -> CurveShape<i32> {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let ellipse = EllipseFrame {
            center,
            axis_x: ArcVector { x: 100, y: 0 },
            axis_y: ArcVector { x: 0, y: 100 },
        };
        let phases = [
            ArcPhase { cos: one, sin: 0 },
            ArcPhase { cos: 0, sin: one },
            ArcPhase { cos: -one, sin: 0 },
            ArcPhase { cos: 0, sin: -one },
            ArcPhase { cos: one, sin: 0 },
        ];
        let points = [
            IntPoint::new(center.x + 100, center.y),
            IntPoint::new(center.x, center.y + 100),
            IntPoint::new(center.x - 100, center.y),
            IntPoint::new(center.x, center.y - 100),
            IntPoint::new(center.x + 100, center.y),
        ];
        let controls = [
            IntPoint::new(center.x + 100, center.y + 100),
            IntPoint::new(center.x - 100, center.y + 100),
            IntPoint::new(center.x - 100, center.y - 100),
            IntPoint::new(center.x + 100, center.y - 100),
        ];
        let segments = (0..4)
            .map(|index| CurveSegment::Arc {
                arc: ArcSegment {
                    ellipse,
                    control_points: [points[index], controls[index], points[index + 1]],
                    weights: [one, 759_250_125, one],
                    start_phase: phases[index],
                    end_phase: phases[index + 1],
                    direction: ArcDirection::CounterClockwise,
                },
            })
            .collect();

        CurveShape {
            contours: vec![CurvePath {
                start: points[0],
                segments,
            }],
        }
    }

    #[test]
    fn add_shape_builds_curve_edges_from_contour_start() {
        let p0 = IntPoint::new(2, 3);
        let p1 = IntPoint::new(8, 5);
        let shape = CurveShape {
            contours: vec![CurvePath {
                start: p0,
                segments: vec![CurveSegment::Line { to: p1 }, CurveSegment::Line { to: p0 }],
            }],
        };
        let mut overlay = IntCurveOverlay {
            curve_slices: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Subject);

        assert_eq!(overlay.curve_slices.len(), 2);
        assert_eq!(overlay.curve_edges.len(), 2);
        assert_eq!(overlay.curve_edges[0].curve_id, CurveId(0));
        assert_eq!(overlay.curve_edges[1].curve_id, CurveId(1));
        assert_eq!(overlay.curve_slices[0].shape_type, ShapeType::Subject);
        match overlay.curve_edges[0].curve {
            Segment::Line(line) => assert_eq!(line.control_points, [p0, p1]),
            _ => panic!("expected line segment"),
        }
        match overlay.curve_edges[1].curve {
            Segment::Line(line) => assert_eq!(line.control_points, [p1, p0]),
            _ => panic!("expected line segment"),
        }
    }

    #[test]
    fn add_shape_canonicalizes_segments_and_preserves_shape_type() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(4, 0);
        let shape = CurveShape {
            contours: vec![CurvePath {
                start: p0,
                segments: vec![
                    CurveSegment::Line { to: p0 },
                    CurveSegment::Quad {
                        ctrl: IntPoint::new(2, 0),
                        to: p1,
                    },
                    CurveSegment::Line { to: p0 },
                ],
            }],
        };
        let mut overlay = IntCurveOverlay {
            curve_slices: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Clip);

        assert_eq!(overlay.curve_slices.len(), 2);
        assert_eq!(overlay.curve_edges.len(), 2);
        for edge in &overlay.curve_edges {
            assert_eq!(overlay.curve_slices[edge.curve_id.0].shape_type, ShapeType::Clip);
            assert!(matches!(edge.curve, Segment::Line(_)));
        }
    }

    #[test]
    fn canonical_edges_keep_their_simple_curve_id() {
        let p0 = IntPoint::new(0, 0);
        let p1 = IntPoint::new(0, -2);
        let shape = CurveShape {
            contours: vec![CurvePath {
                start: p0,
                segments: vec![
                    CurveSegment::Quad {
                        ctrl: IntPoint::new(2, 1),
                        to: p1,
                    },
                    CurveSegment::Line { to: p0 },
                ],
            }],
        };
        let mut overlay = IntCurveOverlay {
            curve_slices: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Subject);

        assert_eq!(overlay.curve_slices.len(), 2);
        let quad_edges: Vec<_> = overlay
            .curve_edges
            .iter()
            .filter(|edge| edge.curve_id == CurveId(0))
            .collect();
        assert_eq!(quad_edges.len(), 2);
        assert!(
            quad_edges
                .iter()
                .all(|edge| matches!(edge.curve, Segment::Quad(_)))
        );
        assert_eq!(overlay.curve_slices[0].marks.len(), 3);
        for edge in quad_edges {
            let chord = edge.curve.chord();
            assert!(overlay.curve_slices[0].param_at(chord.a).is_some());
            assert!(overlay.curve_slices[0].param_at(chord.b).is_some());
        }
        assert_eq!(overlay.curve_edges.last().unwrap().curve_id, CurveId(1));
    }

    #[test]
    fn cusp_pieces_get_distinct_curve_ids() {
        let p0 = IntPoint::new(0, 0);
        let p3 = IntPoint::new(100, 0);
        let shape = CurveShape {
            contours: vec![CurvePath {
                start: p0,
                segments: vec![
                    CurveSegment::Cubic {
                        ctrl0: IntPoint::new(100, 100),
                        ctrl1: IntPoint::new(0, 100),
                        to: p3,
                    },
                    CurveSegment::Line { to: p0 },
                ],
            }],
        };
        let mut overlay = IntCurveOverlay {
            curve_slices: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Subject);

        assert_eq!(overlay.curve_slices.len(), 3);
        assert!(matches!(overlay.curve_slices[0].curve, Segment::Cubic(_)));
        assert!(matches!(overlay.curve_slices[1].curve, Segment::Cubic(_)));
        assert!(matches!(overlay.curve_slices[2].curve, Segment::Line(_)));
        for id in 0..overlay.curve_slices.len() {
            assert!(
                overlay
                    .curve_edges
                    .iter()
                    .any(|edge| edge.curve_id == CurveId(id))
            );
        }
    }

    #[test]
    fn self_intersection_pieces_get_distinct_curve_ids() {
        let p0 = IntPoint::new(0, 0);
        let p3 = IntPoint::new(-14, -14);
        let shape = CurveShape {
            contours: vec![CurvePath {
                start: p0,
                segments: vec![
                    CurveSegment::Cubic {
                        ctrl0: IntPoint::new(-21, -21),
                        ctrl1: IntPoint::new(-21, -14),
                        to: p3,
                    },
                    CurveSegment::Line { to: p0 },
                ],
            }],
        };
        let mut overlay = IntCurveOverlay {
            curve_slices: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Clip);

        assert_eq!(overlay.curve_slices.len(), 3);
        assert!(matches!(overlay.curve_slices[2].curve, Segment::Line(_)));
        for id in 0..overlay.curve_slices.len() {
            assert!(
                overlay
                    .curve_edges
                    .iter()
                    .any(|edge| edge.curve_id == CurveId(id))
            );
        }
    }

    #[test]
    fn path_segments_restore_all_control_points() {
        let p0 = IntPoint::new(1, 2);
        let p1 = IntPoint::new(3, 4);
        let p2 = IntPoint::new(5, 6);
        let p3 = IntPoint::new(7, 8);

        let (quad, end) = CurveSegment::Quad { ctrl: p1, to: p2 }.into_kernel_segment(p0);
        assert_eq!(end, p2);
        match quad {
            Segment::Quad(quad) => assert_eq!(quad.control_points, [p0, p1, p2]),
            _ => panic!("expected quad segment"),
        }

        let (cubic, end) = CurveSegment::Cubic {
            ctrl0: p1,
            ctrl1: p2,
            to: p3,
        }
        .into_kernel_segment(p0);
        assert_eq!(end, p3);
        match cubic {
            Segment::Cubic(cubic) => assert_eq!(cubic.control_points, [p0, p1, p2, p3]),
            _ => panic!("expected cubic segment"),
        }
    }

    #[test]
    fn coincident_chords_merge_curve_ids_in_edge_overlay() {
        fn square() -> CurveShape<i32> {
            let p0 = IntPoint::new(0, 0);
            CurveShape {
                contours: vec![CurvePath {
                    start: p0,
                    segments: vec![
                        CurveSegment::Line {
                            to: IntPoint::new(10, 0),
                        },
                        CurveSegment::Line {
                            to: IntPoint::new(10, 10),
                        },
                        CurveSegment::Line {
                            to: IntPoint::new(0, 10),
                        },
                        CurveSegment::Line { to: p0 },
                    ],
                }],
            }
        }

        let mut overlay = IntCurveOverlay {
            curve_slices: Vec::new(),
            curve_edges: Vec::new(),
        };
        overlay.add_shape(square(), ShapeType::Subject);
        overlay.add_shape(square(), ShapeType::Clip);
        overlay.prepare();

        for edge in &overlay.curve_edges {
            let chord = edge.curve.chord();
            let slice = &overlay.curve_slices[edge.curve_id.0];
            assert!(slice.param_at(chord.a).is_some());
            assert!(slice.param_at(chord.b).is_some());
        }

        let (shapes, store) = overlay.build_vector_shapes(OverlayRule::Intersect, FillRule::NonZero);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].len(), 1);
        assert_eq!(shapes[0][0].len(), 4);

        let mut ids = Vec::new();
        for edge in &shapes[0][0] {
            store.curve_ids(edge.data, &mut ids);
            assert_eq!(ids.len(), 2);
            assert_ne!(ids[0], ids[1]);

            let first_type = overlay.curve_slices[ids[0].0].shape_type;
            let second_type = overlay.curve_slices[ids[1].0].shape_type;
            assert_ne!(first_type, second_type);
        }

        let result = CurveRecomposer::new().recompose(shapes, &store, &overlay.curve_slices);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].contours.len(), 1);
        assert_eq!(result[0].contours[0].segments.len(), 4);

        let public_result = overlay.overlay(OverlayRule::Intersect, FillRule::NonZero);
        assert_eq!(public_result.len(), 1);
        assert_eq!(public_result[0].contours[0].segments.len(), 4);
    }

    #[test]
    fn identical_circles_survive_boolean_intersection_as_arcs() {
        let mut overlay = IntCurveOverlay::new(8);
        overlay.add_shape(circle(IntPoint::new(0, 0)), ShapeType::Subject);
        overlay.add_shape(circle(IntPoint::new(0, 0)), ShapeType::Clip);

        let result = overlay.overlay(OverlayRule::Intersect, FillRule::NonZero);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].contours.len(), 1);
        assert_eq!(result[0].contours[0].segments.len(), 4);
        assert!(
            result[0].contours[0]
                .segments
                .iter()
                .all(|segment| matches!(segment, CurveSegment::Arc { .. }))
        );
    }

    #[test]
    fn overlapping_circles_recompose_split_boundaries_as_arcs() {
        let mut overlay = IntCurveOverlay::new(8);
        overlay.add_shape(circle(IntPoint::new(0, 0)), ShapeType::Subject);
        overlay.add_shape(circle(IntPoint::new(100, 0)), ShapeType::Clip);

        let result = overlay.overlay(OverlayRule::Intersect, FillRule::NonZero);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].contours.len(), 1);
        assert_eq!(result[0].contours[0].segments.len(), 4);
        assert!(
            result[0].contours[0]
                .segments
                .iter()
                .all(|segment| matches!(segment, CurveSegment::Arc { .. }))
        );
    }
}
