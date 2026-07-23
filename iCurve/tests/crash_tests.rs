#[cfg(test)]
mod tests {
    use i_curve::int::bool::overlay::IntCurveOverlay;
    use i_curve::int::curve::path::CurvePath;
    use i_curve::int::curve::segment::CurveSegment;
    use i_curve::int::curve::shape::CurveShape;
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::core::overlay_rule::OverlayRule;
    use i_overlay::i_shape::int::IntPoint;

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

        let mut overlay = IntCurveOverlay::new(4);
        for shape in subject {
            overlay.add_shape(shape, ShapeType::Subject);
        }
        for shape in clip {
            overlay.add_shape(shape, ShapeType::Clip);
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

        let mut overlay = IntCurveOverlay::new(4);
        for shape in subject {
            overlay.add_shape(shape, ShapeType::Subject);
        }
        for shape in clip {
            overlay.add_shape(shape, ShapeType::Clip);
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

        let mut overlay = IntCurveOverlay::new(4);
        for shape in subject {
            overlay.add_shape(shape, ShapeType::Subject);
        }
        for shape in clip {
            overlay.add_shape(shape, ShapeType::Clip);
        }
        let result = overlay.overlay(OverlayRule::Union, FillRule::NonZero);
        dbg!(result);
    }
}
