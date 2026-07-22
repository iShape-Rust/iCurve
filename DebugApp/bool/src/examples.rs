use i_curve::int::curve::{path::CurvePath, segment::CurveSegment, shape::CurveShape};
use i_overlay::i_shape::int::IntPoint;

pub type CurvePoint = IntPoint<i32>;

#[derive(Clone)]
pub struct BoolExample {
    pub name: &'static str,
    pub subject: Vec<CurveShape<i32>>,
    pub clip: Vec<CurveShape<i32>>,
}

pub fn load_examples() -> Vec<BoolExample> {
    vec![
        rect_x_rect(),
        ellipse_x_rect(),
        quad_x_quad(),
        cubic_blobs(),
        two_subjects(),
    ]
}

fn rect_x_rect() -> BoolExample {
    BoolExample {
        name: "rect x rect",
        subject: vec![polygon(&[
            point(-210, -130),
            point(70, -130),
            point(70, 130),
            point(-210, 130),
        ])],
        clip: vec![polygon(&[
            point(-70, -170),
            point(210, -170),
            point(210, 90),
            point(-70, 90),
        ])],
    }
}

fn ellipse_x_rect() -> BoolExample {
    BoolExample {
        name: "cubic ellipse x rect",
        subject: vec![ellipse(point(-55, 0), 155, 125)],
        clip: vec![polygon(&[
            point(-20, -160),
            point(205, -80),
            point(145, 150),
            point(-85, 120),
        ])],
    }
}

fn quad_x_quad() -> BoolExample {
    BoolExample {
        name: "quad x quad",
        subject: vec![CurveShape {
            contours: vec![CurvePath {
                start: point(-200, 0),
                segments: vec![
                    CurveSegment::Line { to: point(200, 0) },
                    CurveSegment::Quad {
                        ctrl: point(0, 240),
                        to: point(-200, 0),
                    },
                ],
            }],
        }],
        clip: vec![CurveShape {
            contours: vec![CurvePath {
                start: point(-110, 55),
                segments: vec![
                    CurveSegment::Line { to: point(110, 55) },
                    CurveSegment::Quad {
                        ctrl: point(0, -245),
                        to: point(-110, 55),
                    },
                ],
            }],
        }],
    }
}

fn cubic_blobs() -> BoolExample {
    BoolExample {
        name: "cubic blobs",
        subject: vec![blob(
            point(-210, 10),
            point(-130, -175),
            point(100, -145),
            point(170, 5),
            point(120, 160),
            point(-135, 165),
        )],
        clip: vec![blob(
            point(-145, -40),
            point(-15, -205),
            point(195, -100),
            point(170, 65),
            point(20, 205),
            point(-205, 95),
        )],
    }
}

fn two_subjects() -> BoolExample {
    BoolExample {
        name: "two subjects",
        subject: vec![
            ellipse(point(-130, -35), 105, 105),
            ellipse(point(45, 35), 105, 105),
        ],
        clip: vec![polygon(&[
            point(-85, -160),
            point(225, -110),
            point(160, 150),
            point(-160, 135),
        ])],
    }
}

fn polygon(points: &[CurvePoint]) -> CurveShape<i32> {
    assert!(points.len() >= 3, "polygon needs at least three points");

    let mut segments = points[1..]
        .iter()
        .copied()
        .map(|to| CurveSegment::Line { to })
        .collect::<Vec<_>>();
    segments.push(CurveSegment::Line { to: points[0] });

    CurveShape {
        contours: vec![CurvePath {
            start: points[0],
            segments,
        }],
    }
}

fn ellipse(center: CurvePoint, radius_x: i32, radius_y: i32) -> CurveShape<i32> {
    // Cubic approximation of a quarter circle: 4 * (sqrt(2) - 1) / 3.
    let control_x = (radius_x as f64 * 0.552_284_749_8).round() as i32;
    let control_y = (radius_y as f64 * 0.552_284_749_8).round() as i32;
    let left = center.x - radius_x;
    let right = center.x + radius_x;
    let bottom = center.y - radius_y;
    let top = center.y + radius_y;

    CurveShape {
        contours: vec![CurvePath {
            start: point(right, center.y),
            segments: vec![
                CurveSegment::Cubic {
                    ctrl0: point(right, center.y + control_y),
                    ctrl1: point(center.x + control_x, top),
                    to: point(center.x, top),
                },
                CurveSegment::Cubic {
                    ctrl0: point(center.x - control_x, top),
                    ctrl1: point(left, center.y + control_y),
                    to: point(left, center.y),
                },
                CurveSegment::Cubic {
                    ctrl0: point(left, center.y - control_y),
                    ctrl1: point(center.x - control_x, bottom),
                    to: point(center.x, bottom),
                },
                CurveSegment::Cubic {
                    ctrl0: point(center.x + control_x, bottom),
                    ctrl1: point(right, center.y - control_y),
                    to: point(right, center.y),
                },
            ],
        }],
    }
}

fn blob(
    p0: CurvePoint,
    p1: CurvePoint,
    p2: CurvePoint,
    p3: CurvePoint,
    p4: CurvePoint,
    p5: CurvePoint,
) -> CurveShape<i32> {
    CurveShape {
        contours: vec![CurvePath {
            start: p0,
            segments: vec![
                CurveSegment::Cubic {
                    ctrl0: p1,
                    ctrl1: p2,
                    to: p3,
                },
                CurveSegment::Cubic {
                    ctrl0: p4,
                    ctrl1: p5,
                    to: p0,
                },
            ],
        }],
    }
}

fn point(x: i32, y: i32) -> CurvePoint {
    IntPoint::new(x, y)
}
