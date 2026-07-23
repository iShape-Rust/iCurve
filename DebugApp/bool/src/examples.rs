use i_curve::float::curve::{
    arc::{Ellipse, EllipticArc},
    builder::CurveBuilder,
    shape::CurveShape,
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
        ellipse_x_rect(),
        quad_x_quad(),
        cubic_blobs(),
        two_subjects(),
        arc_x_arc(),
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

fn arc_x_arc() -> BoolExample {
    BoolExample {
        name: "rotated arc ellipses",
        subject: vec![ellipse([-65.0, 0.0], 175.0, 95.0, 0.38)],
        clip: vec![ellipse([75.0, 5.0], 145.0, 105.0, -0.47)],
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
