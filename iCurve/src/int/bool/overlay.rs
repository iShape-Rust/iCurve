use crate::int::bool::chord_refine::{ChordTopologyRefiner, RefineOutcome};
use crate::int::bool::edge::CurveEdge;
use crate::int::bool::planarize::CurvePlanarizer;
use crate::int::bool::slice::{CurveId, CurveSlice};
use crate::int::curve::shape::CurveShape;
use crate::kernel::int::normalization::canonical::{PushCanonicalSimpleSegment, PushSimpleSegment};
use alloc::vec::Vec;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::int::number::int::IntNumber;

pub struct IntCurveOverlay<I: IntNumber> {
    pub(crate) curve_slices: Vec<CurveSlice<I>>,
    pub(crate) curve_edges: Vec<CurveEdge<I>>,
}

impl<I: IntNumber> IntCurveOverlay<I> {
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
                    self.curve_slices.push(CurveSlice {
                        curve: simple_curve,
                        shape_type,
                    });

                    canonical_curves.clear();
                    canonical_curves.push_canonical_simple(simple_curve);
                    self.curve_edges.extend(
                        canonical_curves
                            .drain(..)
                            .map(|curve| CurveEdge { curve, curve_id }),
                    );
                }

                current = end;
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
        // add_shape already stores simple CurveSlices and converts them into canonical CurveEdges with stable CurveIds

        let mut planarizer = CurvePlanarizer::new();
        let mut chord_refiner = ChordTopologyRefiner::new();

        loop {
            planarizer.planarize(&mut self.curve_edges);

            match chord_refiner.refine(&mut self.curve_edges) {
                RefineOutcome::PlanarityPreserved => break,
                RefineOutcome::Replanarize { escaped_marks } => {
                    #[cfg(all(debug_assertions, feature = "std"))]
                    std::eprintln!(
                        "ChordTopologyRefiner: {escaped_marks} marks escaped their source hull; running CurvePlanarizer again"
                    );

                    #[cfg(not(all(debug_assertions, feature = "std")))]
                    let _ = escaped_marks;
                }
            }
        }

        // 2. all prepared segments can now successfully use EdgeOverlay as a segment data we will use id/or index to shape id/index

        // 3. by EdgeOverlay result segments collect curve slices if neighbor segments has same id it's a part of the same curve slice we join it

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int::curve::path::CurvePath;
    use crate::int::curve::segment::CurveSegment;
    use crate::kernel::int::curve::segment::Segment;
    use alloc::vec;
    use i_overlay::i_shape::int::IntPoint;

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
}
