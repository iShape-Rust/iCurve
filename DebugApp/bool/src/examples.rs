use i_curve::{
    CurveBuilder, FloatCurveShape as CurveShape,
    float::arc::{Ellipse, EllipticArc},
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub type CurvePoint = [f32; 2];

#[derive(Clone)]
pub struct BoolExample {
    pub name: &'static str,
    pub subject: Vec<CurveShape<CurvePoint>>,
    pub clip: Vec<CurveShape<CurvePoint>>,
}

pub fn load_examples() -> Vec<BoolExample> {
    vec![
        rect_x_rect(),
        diamond_x_star(),
        ellipse_x_rect(),
        quad_x_quad(),
        quad_lens_x_capsule(),
        cubic_blobs(),
        cubic_x_ellipse(),
        two_subjects(),
        three_subjects(),
        annulus_x_rect(),
        arc_x_arc(),
        tangent_ellipses(),
        arc_capsule(),
    ]
}

fn rect_x_rect() -> BoolExample {
    BoolExample {
        name: "rect x rect",
        subject: vec![polygon(&[
            [-210.0, -130.0],
            [70.0, -130.0],
            [70.0, 130.0],
            [-210.0, 130.0],
        ])],
        clip: vec![polygon(&[
            [-70.0, -170.0],
            [210.0, -170.0],
            [210.0, 90.0],
            [-70.0, 90.0],
        ])],
    }
}

fn diamond_x_star() -> BoolExample {
    BoolExample {
        name: "diamond x star",
        subject: vec![polygon(&[
            [-190.0, 0.0],
            [0.0, -165.0],
            [190.0, 0.0],
            [0.0, 165.0],
        ])],
        clip: vec![star([35.0, 0.0], 190.0, 78.0, 7, -FRAC_PI_2)],
    }
}

fn ellipse_x_rect() -> BoolExample {
    BoolExample {
        name: "arc ellipse x rect",
        subject: vec![ellipse([-55.0, 0.0], 155.0, 125.0, 0.0)],
        clip: vec![polygon(&[
            [-20.0, -160.0],
            [205.0, -80.0],
            [145.0, 150.0],
            [-85.0, 120.0],
        ])],
    }
}

fn quad_x_quad() -> BoolExample {
    let subject = CurveBuilder::new()
        .move_to([-200.0, 0.0])
        .unwrap()
        .line_to([200.0, 0.0])
        .unwrap()
        .quad_to([0.0, 240.0], [-200.0, 0.0])
        .unwrap()
        .build()
        .unwrap();
    let clip = CurveBuilder::new()
        .move_to([-110.0, 55.0])
        .unwrap()
        .line_to([110.0, 55.0])
        .unwrap()
        .quad_to([0.0, -245.0], [-110.0, 55.0])
        .unwrap()
        .build()
        .unwrap();

    BoolExample {
        name: "quad x quad",
        subject: vec![subject],
        clip: vec![clip],
    }
}

fn quad_lens_x_capsule() -> BoolExample {
    BoolExample {
        name: "quad lens x capsule",
        subject: vec![quad_lens(
            [-210.0, 0.0],
            [190.0, 0.0],
            [0.0, -245.0],
            [0.0, 245.0],
        )],
        clip: vec![capsule([-120.0, -45.0], [115.0, 65.0], 64.0)],
    }
}

fn cubic_blobs() -> BoolExample {
    BoolExample {
        name: "cubic blobs",
        subject: vec![blob(
            [-210.0, 10.0],
            [-130.0, -175.0],
            [100.0, -145.0],
            [170.0, 5.0],
            [120.0, 160.0],
            [-135.0, 165.0],
        )],
        clip: vec![blob(
            [-145.0, -40.0],
            [-15.0, -205.0],
            [195.0, -100.0],
            [170.0, 65.0],
            [20.0, 205.0],
            [-205.0, 95.0],
        )],
    }
}

fn cubic_x_ellipse() -> BoolExample {
    BoolExample {
        name: "cubic blob x arc ellipse",
        subject: vec![blob(
            [-215.0, 15.0],
            [-155.0, -210.0],
            [135.0, -185.0],
            [195.0, 10.0],
            [100.0, 205.0],
            [-165.0, 190.0],
        )],
        clip: vec![ellipse([25.0, -5.0], 145.0, 90.0, 0.62)],
    }
}

fn two_subjects() -> BoolExample {
    BoolExample {
        name: "two arc subjects",
        subject: vec![
            ellipse([-130.0, -35.0], 105.0, 105.0, 0.0),
            ellipse([45.0, 35.0], 105.0, 105.0, 0.0),
        ],
        clip: vec![polygon(&[
            [-85.0, -160.0],
            [225.0, -110.0],
            [160.0, 150.0],
            [-160.0, 135.0],
        ])],
    }
}

fn three_subjects() -> BoolExample {
    BoolExample {
        name: "three subjects x rect",
        subject: vec![
            ellipse([-135.0, -70.0], 82.0, 82.0, 0.0),
            ellipse([0.0, 75.0], 105.0, 105.0, 0.0),
            ellipse([135.0, -55.0], 92.0, 92.0, 0.0),
        ],
        clip: vec![polygon(&[
            [-85.0, -190.0],
            [95.0, -190.0],
            [95.0, 190.0],
            [-85.0, 190.0],
        ])],
    }
}

fn annulus_x_rect() -> BoolExample {
    BoolExample {
        name: "arc annulus x rect",
        subject: vec![annulus([-20.0, 0.0], 190.0, 92.0)],
        clip: vec![polygon(&[
            [-225.0, -70.0],
            [210.0, -120.0],
            [225.0, 65.0],
            [-205.0, 120.0],
        ])],
    }
}

fn arc_x_arc() -> BoolExample {
    BoolExample {
        name: "rotated arc ellipses",
        subject: vec![ellipse([-65.0, 0.0], 175.0, 95.0, 0.38)],
        clip: vec![ellipse([75.0, 5.0], 145.0, 105.0, -0.47)],
    }
}

fn tangent_ellipses() -> BoolExample {
    BoolExample {
        name: "tangent arc circles",
        subject: vec![ellipse([-110.0, 0.0], 110.0, 110.0, 0.0)],
        clip: vec![ellipse([110.0, 0.0], 110.0, 110.0, 0.0)],
    }
}

fn arc_capsule() -> BoolExample {
    BoolExample {
        name: "arc capsule x ellipse",
        subject: vec![capsule([-175.0, -55.0], [125.0, 55.0], 72.0)],
        clip: vec![ellipse([35.0, 0.0], 135.0, 135.0, 0.0)],
    }
}

fn polygon(points: &[CurvePoint]) -> CurveShape<CurvePoint> {
    assert!(points.len() >= 3, "polygon needs at least three points");

    let mut builder = CurveBuilder::new().move_to(points[0]).unwrap();
    for point in &points[1..] {
        builder = builder.line_to(*point).unwrap();
    }

    builder.close_contour().unwrap().build().unwrap()
}

fn star(
    center: CurvePoint,
    outer_radius: f32,
    inner_radius: f32,
    point_count: usize,
    rotation: f32,
) -> CurveShape<CurvePoint> {
    let vertex_count = point_count * 2;
    let points = (0..vertex_count)
        .map(|index| {
            let angle = rotation + index as f32 * TAU / vertex_count as f32;
            let radius = if index % 2 == 0 {
                outer_radius
            } else {
                inner_radius
            };
            [
                center[0] + radius * angle.cos(),
                center[1] + radius * angle.sin(),
            ]
        })
        .collect::<Vec<_>>();

    polygon(&points)
}

fn quad_lens(
    left: CurvePoint,
    right: CurvePoint,
    top_control: CurvePoint,
    bottom_control: CurvePoint,
) -> CurveShape<CurvePoint> {
    CurveBuilder::new()
        .move_to(left)
        .unwrap()
        .quad_to(top_control, right)
        .unwrap()
        .quad_to(bottom_control, left)
        .unwrap()
        .build()
        .unwrap()
}

fn ellipse(
    center: CurvePoint,
    radius_x: f32,
    radius_y: f32,
    rotation: f32,
) -> CurveShape<CurvePoint> {
    let arc = EllipticArc {
        ellipse: Ellipse {
            center,
            radius_x,
            radius_y,
            rotation,
        },
        start_angle: 0.0,
        sweep_angle: TAU,
    };

    CurveBuilder::new()
        .move_to(arc.start_point())
        .unwrap()
        .arc_to(arc)
        .unwrap()
        .build()
        .unwrap()
}

fn annulus(center: CurvePoint, outer_radius: f32, inner_radius: f32) -> CurveShape<CurvePoint> {
    let outer = EllipticArc {
        ellipse: Ellipse {
            center,
            radius_x: outer_radius,
            radius_y: outer_radius,
            rotation: 0.0,
        },
        start_angle: 0.0,
        sweep_angle: TAU,
    };
    let inner = EllipticArc {
        ellipse: Ellipse {
            center,
            radius_x: inner_radius,
            radius_y: inner_radius,
            rotation: 0.0,
        },
        start_angle: 0.0,
        sweep_angle: -TAU,
    };

    CurveBuilder::new()
        .move_to(outer.start_point())
        .unwrap()
        .arc_to(outer)
        .unwrap()
        .move_to(inner.start_point())
        .unwrap()
        .arc_to(inner)
        .unwrap()
        .build()
        .unwrap()
}

fn blob(
    p0: CurvePoint,
    p1: CurvePoint,
    p2: CurvePoint,
    p3: CurvePoint,
    p4: CurvePoint,
    p5: CurvePoint,
) -> CurveShape<CurvePoint> {
    CurveBuilder::new()
        .move_to(p0)
        .unwrap()
        .cubic_to(p1, p2, p3)
        .unwrap()
        .cubic_to(p4, p5, p0)
        .unwrap()
        .build()
        .unwrap()
}

fn capsule(
    left_center: CurvePoint,
    right_center: CurvePoint,
    radius: f32,
) -> CurveShape<CurvePoint> {
    let right_arc = EllipticArc {
        ellipse: Ellipse {
            center: right_center,
            radius_x: radius,
            radius_y: radius,
            rotation: 0.0,
        },
        start_angle: -FRAC_PI_2,
        sweep_angle: PI,
    };
    let left_arc = EllipticArc {
        ellipse: Ellipse {
            center: left_center,
            radius_x: radius,
            radius_y: radius,
            rotation: 0.0,
        },
        start_angle: FRAC_PI_2,
        sweep_angle: PI,
    };

    CurveBuilder::new()
        .move_to(right_arc.start_point())
        .unwrap()
        .arc_to(right_arc)
        .unwrap()
        .line_to(left_arc.start_point())
        .unwrap()
        .arc_to(left_arc)
        .unwrap()
        .close_contour()
        .unwrap()
        .build()
        .unwrap()
}
