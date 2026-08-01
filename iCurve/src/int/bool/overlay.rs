use crate::int::CURVE_COORDINATE_SAFETY_BITS;
use crate::int::CurveInt;
use crate::int::bool::approximate::CurveApproximator;
use crate::int::bool::data::{CurveEdgeData, CurveEdgeDataStore, CurveSourceSpan};
use crate::int::bool::edge::CurveEdge;
use crate::int::bool::planarize::CurvePlanarizer;
use crate::int::bool::recompose::CurveRecomposer;
use crate::int::bool::source::{CurveId, CurveSource};
use crate::int::curve::shape::CurveShape;
use crate::kernel::int::curve::arc::RationalArcError;
use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::normalization::canonical::{PushCanonicalSimpleParametricSegment, PushSimpleSegment};
use alloc::vec::Vec;
use i_overlay::core::edge_overlay::{EdgeOverlay, InputEdge};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::core::solver::Solver;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::vector::edge::DataVectorShape;

/// Structural error in an integer curve input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CurveInputError {
    /// A shape has no contours.
    EmptyShape,
    /// A contour has no segments.
    EmptyContour {
        /// Zero-based contour index within the rejected shape.
        contour: usize,
    },
    /// The last endpoint of a contour does not equal its start point.
    UnclosedContour {
        /// Zero-based contour index within the rejected shape.
        contour: usize,
    },
    /// A rational arc does not start at the preceding segment endpoint.
    DisconnectedArc {
        /// Zero-based contour index within the rejected shape.
        contour: usize,
        /// Zero-based segment index within the rejected contour.
        segment: usize,
    },
    /// A rational arc violates an integer kernel invariant.
    InvalidArc {
        /// Zero-based contour index within the rejected shape.
        contour: usize,
        /// Zero-based segment index within the rejected contour.
        segment: usize,
        /// Specific invalid arc invariant.
        error: RationalArcError,
    },
}

impl core::fmt::Display for CurveInputError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::EmptyShape => formatter.write_str("curve shape has no contours"),
            Self::EmptyContour { contour } => {
                write!(formatter, "curve contour {contour} has no segments")
            }
            Self::UnclosedContour { contour } => {
                write!(formatter, "curve contour {contour} is not closed")
            }
            Self::DisconnectedArc { contour, segment } => write!(
                formatter,
                "rational arc {segment} in contour {contour} is disconnected"
            ),
            Self::InvalidArc {
                contour,
                segment,
                error: _,
            } => write!(
                formatter,
                "rational arc {segment} in contour {contour} is invalid"
            ),
        }
    }
}

impl core::error::Error for CurveInputError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidArc { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// Invalid [`CurveOverlayOptions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CurveOverlayOptionsError {
    /// The requested subdivision limit exceeds the library safety ceiling.
    MaxApproximationDepthTooLarge {
        /// Value supplied by the caller.
        requested: u32,
        /// Largest value accepted by this version of the library.
        maximum: u32,
    },
}

impl core::fmt::Display for CurveOverlayOptionsError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::MaxApproximationDepthTooLarge { requested, maximum } => write!(
                formatter,
                "maximum approximation depth {requested} exceeds the safety limit {maximum}"
            ),
        }
    }
}

impl core::error::Error for CurveOverlayOptionsError {}

/// Controls the chord approximation used to determine boolean topology.
///
/// These values are expressed in the integer coordinate system. The default
/// is intended for general-purpose input; change it only after selecting the
/// integer or float conversion scale deliberately.
///
/// Construct this non-exhaustive configuration from [`Default`] and override
/// only the values your application needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CurveOverlayOptions {
    /// Chords shorter than `2^min_chord_length_power` are accepted without
    /// further approximation subdivision.
    pub min_chord_length_power: u32,
    /// A segment is accepted when its endpoint handles are collinear with its
    /// chord within a sine tolerance of approximately `2^-angle_tolerance_power`.
    pub angle_tolerance_power: u32,
    /// Hard safety limit for local approximation subdivision. Values above
    /// [`MAX_APPROXIMATION_DEPTH`](Self::MAX_APPROXIMATION_DEPTH) are rejected.
    pub max_approximation_depth: u32,
}

impl Default for CurveOverlayOptions {
    fn default() -> Self {
        Self {
            min_chord_length_power: 4,
            angle_tolerance_power: 3,
            max_approximation_depth: Self::MAX_APPROXIMATION_DEPTH,
        }
    }
}

impl CurveOverlayOptions {
    /// Absolute safety ceiling for local approximation subdivision.
    pub const MAX_APPROXIMATION_DEPTH: u32 = 16;

    /// Sets the minimum accepted chord length power.
    #[must_use]
    pub const fn with_min_chord_length_power(mut self, power: u32) -> Self {
        self.min_chord_length_power = power;
        self
    }

    /// Sets the angle tolerance power.
    #[must_use]
    pub const fn with_angle_tolerance_power(mut self, power: u32) -> Self {
        self.angle_tolerance_power = power;
        self
    }

    /// Sets the local approximation subdivision limit.
    #[must_use]
    pub const fn with_max_approximation_depth(mut self, depth: u32) -> Self {
        self.max_approximation_depth = depth;
        self
    }

    /// Validates the computational safety limits of this configuration.
    pub fn validate(&self) -> Result<(), CurveOverlayOptionsError> {
        if self.max_approximation_depth > Self::MAX_APPROXIMATION_DEPTH {
            return Err(CurveOverlayOptionsError::MaxApproximationDepthTooLarge {
                requested: self.max_approximation_depth,
                maximum: Self::MAX_APPROXIMATION_DEPTH,
            });
        }

        Ok(())
    }
}

/// Incremental Boolean overlay for integer curve shapes.
///
/// Shapes are validated when passed to [`add_subject`](Self::add_subject),
/// [`add_clip`](Self::add_clip), or [`add_shape`](Self::add_shape). Add any
/// number of inputs, configure the solver and approximation, then consume the
/// builder with [`overlay`](Self::overlay).
pub struct IntCurveOverlay<I: CurveInt> {
    solver: Solver,
    options: CurveOverlayOptions,
    pub(crate) curve_sources: Vec<CurveSource<I>>,
    pub(crate) curve_edges: Vec<CurveEdge<I>>,
}

impl<I: CurveInt> IntCurveOverlay<I> {
    /// Creates an empty overlay.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates an empty overlay with an input-segment allocation hint.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            solver: Solver::default(),
            options: CurveOverlayOptions::default(),
            curve_sources: Vec::with_capacity(capacity),
            curve_edges: Vec::with_capacity(capacity),
        }
    }

    /// Sets the polygon solver strategy and precision.
    #[must_use]
    pub fn with_solver(mut self, solver: Solver) -> Self {
        self.solver = solver;
        self
    }

    /// Validates and sets the curve approximation options.
    pub fn try_with_options(
        mut self,
        options: CurveOverlayOptions,
    ) -> Result<Self, CurveOverlayOptionsError> {
        options.validate()?;
        self.options = options;
        Ok(self)
    }

    /// Returns the polygon solver configuration.
    #[inline]
    pub fn solver(&self) -> Solver {
        self.solver
    }

    /// Returns the curve approximation configuration.
    #[inline]
    pub fn options(&self) -> CurveOverlayOptions {
        self.options
    }

    /// Adds a subject shape.
    #[inline]
    pub fn add_subject(&mut self, shape: CurveShape<I>) -> Result<(), CurveInputError> {
        self.add_shape(shape, ShapeType::Subject)
    }

    /// Adds a clip shape.
    #[inline]
    pub fn add_clip(&mut self, shape: CurveShape<I>) -> Result<(), CurveInputError> {
        self.add_shape(shape, ShapeType::Clip)
    }

    /// Validates and adds a shape as a subject or clip operand.
    pub fn add_shape(&mut self, shape: CurveShape<I>, shape_type: ShapeType) -> Result<(), CurveInputError> {
        validate_shape(&shape)?;
        let mut simple_curves = Vec::new();
        let mut canonical_curves = Vec::new();

        for contour in shape.contours {
            let mut current = contour.start;

            for segment in contour.segments {
                let (curve, end) = segment.into_kernel_segment(current);
                simple_curves.clear();
                simple_curves.push_simple(curve);

                for simple_curve in simple_curves.drain(..) {
                    let curve_id = CurveId(self.curve_sources.len());
                    canonical_curves.clear();
                    canonical_curves.push_canonical_simple_parametric(simple_curve);

                    self.curve_sources
                        .push(CurveSource::new(simple_curve, shape_type));

                    self.curve_edges
                        .extend(canonical_curves.drain(..).map(|canonical| {
                            CurveEdge::new(canonical.curve, curve_id, canonical.start, canonical.end)
                        }));
                }

                current = end;
            }
        }

        Ok(())
    }

    fn prepare(&mut self) {
        // Build a bounded local chord approximation first, then run exactly one
        // curve-aware planarization pass for nearby/intersecting curve pieces.
        CurveApproximator::new().approximate(&mut self.curve_edges, self.options);

        let mut planarizer = CurvePlanarizer::new();
        let cross_radius = self.initial_snap_radius();
        planarizer.planarize(&mut self.curve_edges, cross_radius);
    }

    #[inline]
    fn initial_snap_radius(&self) -> I::Wide {
        let coordinate_bits = I::BITS - CURVE_COORDINATE_SAFETY_BITS;
        let max_exponent = 2 * coordinate_bits;
        I::Wide::ONE << (self.solver.precision.start as u32).min(max_exponent)
    }

    fn build_vector_shapes(
        &self,
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> (Vec<DataVectorShape<I, CurveEdgeData>>, CurveEdgeDataStore) {
        let mut edge_overlay = EdgeOverlay::new(self.curve_edges.len());
        edge_overlay.solver = self.solver;

        for edge in &self.curve_edges {
            let chord = edge.curve.chord();
            let curve_source = &self.curve_sources[edge.curve_id.0];
            edge_overlay.add_edge(
                InputEdge {
                    a: chord.a,
                    b: chord.b,
                    data: CurveEdgeData::Single(CurveSourceSpan::from_edge(*edge)),
                },
                curve_source.shape_type,
            );
        }

        let shapes = edge_overlay.build_vector_shapes(overlay_rule, fill_rule);
        let data_store = edge_overlay.into_data_store();

        (shapes, data_store)
    }

    /// Resolves the configured Boolean operation and returns reconstructed curves.
    #[inline]
    pub fn overlay(mut self, overlay_rule: OverlayRule, fill_rule: FillRule) -> Vec<CurveShape<I>> {
        self.prepare();

        // Resolve the boolean topology while preserving CurveId provenance.
        let (vector_shapes, data_store) = self.build_vector_shapes(overlay_rule, fill_rule);

        // Restore maximal runs from their source curves and parameter spans.
        CurveRecomposer::new().recompose(vector_shapes, &data_store, &self.curve_sources)
    }
}

impl<I: CurveInt> Default for IntCurveOverlay<I> {
    fn default() -> Self {
        Self::new()
    }
}

/// Performs a boolean operation on one subject and one clip shape.
///
/// Use [`IntCurveOverlay`] when an operation has more inputs or needs custom
/// approximation or polygon solver options.
pub fn overlay<I>(
    subject: CurveShape<I>,
    clip: CurveShape<I>,
    overlay_rule: OverlayRule,
    fill_rule: FillRule,
) -> Result<Vec<CurveShape<I>>, CurveInputError>
where
    I: CurveInt,
{
    let capacity = segment_count(&subject).saturating_add(segment_count(&clip));
    let mut overlay = IntCurveOverlay::with_capacity(capacity);
    overlay.add_subject(subject)?;
    overlay.add_clip(clip)?;
    Ok(overlay.overlay(overlay_rule, fill_rule))
}

fn segment_count<I: CurveInt>(shape: &CurveShape<I>) -> usize {
    shape.contours.iter().map(|contour| contour.segments.len()).sum()
}

fn validate_shape<I: CurveInt>(shape: &CurveShape<I>) -> Result<(), CurveInputError> {
    if shape.contours.is_empty() {
        return Err(CurveInputError::EmptyShape);
    }

    for (contour_index, contour) in shape.contours.iter().enumerate() {
        if contour.segments.is_empty() {
            return Err(CurveInputError::EmptyContour {
                contour: contour_index,
            });
        }

        let mut current = contour.start;
        for (segment_index, segment) in contour.segments.iter().enumerate() {
            current = match segment {
                crate::int::curve::segment::CurveSegment::Line { to }
                | crate::int::curve::segment::CurveSegment::Quad { to, .. }
                | crate::int::curve::segment::CurveSegment::Cubic { to, .. } => *to,
                crate::int::curve::segment::CurveSegment::Arc { arc } => {
                    if arc.control_points[0] != current {
                        return Err(CurveInputError::DisconnectedArc {
                            contour: contour_index,
                            segment: segment_index,
                        });
                    }
                    arc.validate().map_err(|error| CurveInputError::InvalidArc {
                        contour: contour_index,
                        segment: segment_index,
                        error,
                    })?;
                    arc.control_points[2]
                }
            };
        }

        if current != contour.start {
            return Err(CurveInputError::UnclosedContour {
                contour: contour_index,
            });
        }
    }

    Ok(())
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
            solver: Solver::default(),
            options: CurveOverlayOptions::default(),
            curve_sources: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Subject).unwrap();

        assert_eq!(overlay.curve_sources.len(), 2);
        assert_eq!(overlay.curve_edges.len(), 2);
        assert_eq!(overlay.curve_edges[0].curve_id, CurveId(0));
        assert_eq!(overlay.curve_edges[1].curve_id, CurveId(1));
        assert_eq!(overlay.curve_sources[0].shape_type, ShapeType::Subject);
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
            solver: Solver::default(),
            options: CurveOverlayOptions::default(),
            curve_sources: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Clip).unwrap();

        assert_eq!(overlay.curve_sources.len(), 2);
        assert_eq!(overlay.curve_edges.len(), 2);
        for edge in &overlay.curve_edges {
            assert_eq!(overlay.curve_sources[edge.curve_id.0].shape_type, ShapeType::Clip);
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
            solver: Solver::default(),
            options: CurveOverlayOptions::default(),
            curve_sources: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Subject).unwrap();

        assert_eq!(overlay.curve_sources.len(), 2);
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
        assert_eq!(quad_edges[0].start_param.value(), 0_i64);
        assert!(quad_edges[0].end_param.value() <= quad_edges[1].start_param.value());
        assert_eq!(
            quad_edges[1].end_param.value(),
            crate::kernel::int::curve::param::SegmentParam::<i32>::DENOMINATOR
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
            solver: Solver::default(),
            options: CurveOverlayOptions::default(),
            curve_sources: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Subject).unwrap();

        assert_eq!(overlay.curve_sources.len(), 3);
        assert!(matches!(overlay.curve_sources[0].curve, Segment::Cubic(_)));
        assert!(matches!(overlay.curve_sources[1].curve, Segment::Cubic(_)));
        assert!(matches!(overlay.curve_sources[2].curve, Segment::Line(_)));
        for id in 0..overlay.curve_sources.len() {
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
            solver: Solver::default(),
            options: CurveOverlayOptions::default(),
            curve_sources: Vec::new(),
            curve_edges: Vec::new(),
        };

        overlay.add_shape(shape, ShapeType::Clip).unwrap();

        assert_eq!(overlay.curve_sources.len(), 3);
        assert!(matches!(overlay.curve_sources[2].curve, Segment::Line(_)));
        for id in 0..overlay.curve_sources.len() {
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
            solver: Solver::default(),
            options: CurveOverlayOptions::default(),
            curve_sources: Vec::new(),
            curve_edges: Vec::new(),
        };
        overlay.add_shape(square(), ShapeType::Subject).unwrap();
        overlay.add_shape(square(), ShapeType::Clip).unwrap();
        overlay.prepare();

        let (shapes, store) = overlay.build_vector_shapes(OverlayRule::Intersect, FillRule::NonZero);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].len(), 1);
        assert_eq!(shapes[0][0].len(), 4);

        let mut spans = Vec::new();
        for edge in &shapes[0][0] {
            store.spans(edge.data, &mut spans);
            assert_eq!(spans.len(), 2);
            assert_ne!(spans[0].curve_id, spans[1].curve_id);

            let first_type = overlay.curve_sources[spans[0].curve_id.0].shape_type;
            let second_type = overlay.curve_sources[spans[1].curve_id.0].shape_type;
            assert_ne!(first_type, second_type);
        }

        let result = CurveRecomposer::new().recompose(shapes, &store, &overlay.curve_sources);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].contours.len(), 1);
        assert_eq!(result[0].contours[0].segments.len(), 4);

        let public_result = overlay.overlay(OverlayRule::Intersect, FillRule::NonZero);
        assert_eq!(public_result.len(), 1);
        assert_eq!(public_result[0].contours[0].segments.len(), 4);
    }

    #[test]
    fn identical_circles_survive_boolean_intersection_as_arcs() {
        let mut overlay = IntCurveOverlay::with_capacity(8);
        overlay
            .add_shape(circle(IntPoint::new(0, 0)), ShapeType::Subject)
            .unwrap();
        overlay
            .add_shape(circle(IntPoint::new(0, 0)), ShapeType::Clip)
            .unwrap();

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
        let mut overlay = IntCurveOverlay::with_capacity(8);
        overlay
            .add_shape(circle(IntPoint::new(0, 0)), ShapeType::Subject)
            .unwrap();
        overlay
            .add_shape(circle(IntPoint::new(100, 0)), ShapeType::Clip)
            .unwrap();

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
    fn with_solver_preserves_precision_settings() {
        use i_overlay::core::solver::Precision;

        let solver = Solver::with_precision(Precision::LOW);
        let overlay = IntCurveOverlay::<i32>::with_capacity(16).with_solver(solver);

        assert_eq!(overlay.solver.precision, Precision::LOW);
        assert_eq!(overlay.curve_edges.capacity(), 16);
        assert_eq!(overlay.curve_sources.capacity(), 16);
    }

    #[test]
    fn try_with_options_preserves_approximation_settings() {
        let options = CurveOverlayOptions {
            min_chord_length_power: 6,
            angle_tolerance_power: 5,
            max_approximation_depth: 12,
        };

        let overlay = IntCurveOverlay::<i32>::with_capacity(4)
            .try_with_options(options)
            .unwrap();

        assert_eq!(overlay.options, options);
        assert_eq!(overlay.curve_edges.capacity(), 4);
    }
}
