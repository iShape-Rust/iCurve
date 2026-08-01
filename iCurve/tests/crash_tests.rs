#[cfg(test)]
mod tests {
    use i_curve::float::CurveConverter;
    use i_curve::float::arc::{Ellipse, EllipticArc};
    use i_curve::int::{CurvePath, CurveSegment, CurveShape, IntCurveOverlay, IntPoint, ShapeType};
    use i_curve::{CurveBuilder, FillRule, OverlayRule};

    #[test]
    fn test_00() {
        let subject = vec![CurveShape {
            contours: vec![CurvePath {
                start: IntPoint::new(-200, 0),
                segments: vec![
                    CurveSegment::Line {
                        to: IntPoint::new(200, 0),
                    },
                    CurveSegment::Quad {
                        ctrl: IntPoint::new(0, 240),
                        to: IntPoint::new(-200, 0),
                    },
                ],
            }],
        }];

        let clip = vec![CurveShape {
            contours: vec![CurvePath {
                start: IntPoint::new(-110, 55),
                segments: vec![
                    CurveSegment::Line {
                        to: IntPoint::new(110, 55),
                    },
                    CurveSegment::Quad {
                        ctrl: IntPoint::new(-177, -145),
                        to: IntPoint::new(-110, 55),
                    },
                ],
            }],
        }];

        let mut overlay = IntCurveOverlay::with_capacity(4);
        for shape in subject {
            overlay.add_shape(shape, ShapeType::Subject).unwrap();
        }
        for shape in clip {
            overlay.add_shape(shape, ShapeType::Clip).unwrap();
        }
        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);
        dbg!(result);
    }

    #[test]
    fn test_01() {
        let subject = vec![CurveShape {
            contours: vec![CurvePath {
                start: IntPoint::new(-200, 0),
                segments: vec![
                    CurveSegment::Line {
                        to: IntPoint::new(200, 0),
                    },
                    CurveSegment::Quad {
                        ctrl: IntPoint::new(0, 240),
                        to: IntPoint::new(-200, 0),
                    },
                ],
            }],
        }];

        let clip = vec![CurveShape {
            contours: vec![CurvePath {
                start: IntPoint::new(-110, 55),
                segments: vec![
                    CurveSegment::Line {
                        to: IntPoint::new(110, 55),
                    },
                    CurveSegment::Quad {
                        ctrl: IntPoint::new(-27, 193),
                        to: IntPoint::new(-110, 55),
                    },
                ],
            }],
        }];

        let mut overlay = IntCurveOverlay::with_capacity(4);
        for shape in subject {
            overlay.add_shape(shape, ShapeType::Subject).unwrap();
        }
        for shape in clip {
            overlay.add_shape(shape, ShapeType::Clip).unwrap();
        }
        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);
        dbg!(result);
    }

    #[test]
    fn test_02() {
        let subject = vec![CurveShape {
            contours: vec![CurvePath {
                start: IntPoint::new(-210, 10),
                segments: vec![
                    CurveSegment::Cubic {
                        ctrl0: IntPoint::new(-130, -175),
                        ctrl1: IntPoint::new(100, -145),
                        to: IntPoint::new(170, 5),
                    },
                    CurveSegment::Cubic {
                        ctrl0: IntPoint::new(120, 160),
                        ctrl1: IntPoint::new(-135, 165),
                        to: IntPoint::new(-210, 10),
                    },
                ],
            }],
        }];

        let clip = vec![CurveShape {
            contours: vec![CurvePath {
                start: IntPoint::new(-145, -40),
                segments: vec![
                    CurveSegment::Cubic {
                        ctrl0: IntPoint::new(-15, -205),
                        ctrl1: IntPoint::new(195, -100),
                        to: IntPoint::new(170, 65),
                    },
                    CurveSegment::Cubic {
                        ctrl0: IntPoint::new(-192, 115),
                        ctrl1: IntPoint::new(-197, 128),
                        to: IntPoint::new(-145, -40),
                    },
                ],
            }],
        }];

        let mut overlay = IntCurveOverlay::with_capacity(4);
        for shape in subject {
            overlay.add_shape(shape, ShapeType::Subject).unwrap();
        }
        for shape in clip {
            overlay.add_shape(shape, ShapeType::Clip).unwrap();
        }
        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);
        dbg!(result);
    }

    #[test]
    fn test_03() {
        const SCALE: f32 = 1.0;
        const SUBJECT_CONTOURS: usize = 1;

        let input = CurveBuilder::new()
            .move_to([47.376312, 69.676125])
            .unwrap()
            .arc_to(EllipticArc {
                ellipse: Ellipse {
                    center: [-115.14, 4.76504],
                    radius_x: 175.0,
                    radius_y: 95.0,
                    rotation: 0.38,
                },
                start_angle: 0.0,
                sweep_angle: 6.2831855,
            })
            .unwrap()
            .move_to([204.2774, -60.66851])
            .unwrap()
            .arc_to(EllipticArc {
                ellipse: Ellipse {
                    center: [75.0, 5.0],
                    radius_x: 145.0,
                    radius_y: 105.0,
                    rotation: -0.47,
                },
                start_angle: 0.0,
                sweep_angle: 6.2831855,
            })
            .unwrap()
            .build()
            .unwrap();

        let converter = CurveConverter::<_, i32>::try_with_scale(input, SCALE).unwrap();
        let int_shape = converter.into_shape();
        assert_eq!(int_shape.contours.len(), 2);
        let capacity = int_shape
            .contours
            .iter()
            .map(|contour| contour.segments.len())
            .sum();
        let mut overlay = IntCurveOverlay::with_capacity(capacity);
        for (index, contour) in int_shape.contours.into_iter().enumerate() {
            let shape_type = if index < SUBJECT_CONTOURS {
                ShapeType::Subject
            } else {
                ShapeType::Clip
            };
            overlay
                .add_shape(
                    CurveShape {
                        contours: vec![contour],
                    },
                    shape_type,
                )
                .unwrap();
        }
        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);
        dbg!(result);
    }

    /// Minimized from randomized stress case 1. This must complete without runaway refinement.
    #[test]
    fn randomized_case_1_minimized_pathological_refinement() {
        let clip = CurveShape {
            contours: vec![CurvePath {
                start: IntPoint::new(-475_301, -855_672),
                segments: vec![
                    CurveSegment::Cubic {
                        ctrl0: IntPoint::new(-475_301, -855_672),
                        ctrl1: IntPoint::new(5, -2),
                        to: IntPoint::new(-475_303, -855_675),
                    },
                    CurveSegment::Quad {
                        ctrl: IntPoint::new(1, 1),
                        to: IntPoint::new(-475_301, -855_672),
                    },
                ],
            }],
        };

        let mut overlay = IntCurveOverlay::with_capacity(2);
        overlay.add_shape(clip, ShapeType::Clip).unwrap();

        let result = overlay.overlay(OverlayRule::Clip, FillRule::NonZero);
        assert!(!result.is_empty());
    }
}
