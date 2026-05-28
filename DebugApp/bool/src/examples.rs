use i_curve::curve::{arc::EllipticArc, builder::CurveShapeBuilder, shape::CurveShape};
use std::f32::consts::{FRAC_PI_2, PI};

pub type CurvePoint = [f32; 2];

pub struct BoolExample {
    pub name: &'static str,
    pub subject: Vec<CurveShape<CurvePoint>>,
    pub clip: Vec<CurveShape<CurvePoint>>,
}

pub fn load_examples() -> Vec<BoolExample> {
    vec![
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
        },
        BoolExample {
            name: "circle x rect",
            subject: vec![circle([-65.0, 0.0], 150.0)],
            clip: vec![polygon(&[
                [-10.0, -155.0],
                [205.0, -80.0],
                [145.0, 150.0],
                [-85.0, 120.0],
            ])],
        },
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
        },
        BoolExample {
            name: "two subjects",
            subject: vec![circle([-135.0, -35.0], 105.0), circle([45.0, 35.0], 105.0)],
            clip: vec![polygon(&[
                [-85.0, -160.0],
                [225.0, -110.0],
                [160.0, 150.0],
                [-160.0, 135.0],
            ])],
        },
        BoolExample {
            name: "arc capsule",
            subject: vec![capsule([-215.0, -95.0], [135.0, 95.0], 82.0)],
            clip: vec![circle([45.0, 0.0], 135.0)],
        },
        BoolExample {
            name: "cubic solo",
            subject: vec![scaled_cubic_solo(150.0)],
            clip: Vec::new(),
        },
    ]
}

fn polygon(points: &[CurvePoint]) -> CurveShape<CurvePoint> {
    assert!(points.len() >= 3, "polygon needs at least three points");

    let mut builder = CurveShapeBuilder::new()
        .move_to(points[0])
        .expect("move_to");

    for point in &points[1..] {
        builder = builder.line_to(*point).expect("line_to");
    }

    builder
        .line_to(points[0])
        .expect("line_to")
        .build()
        .expect("polygon")
}

fn circle(center: CurvePoint, radius: f32) -> CurveShape<CurvePoint> {
    ellipse(center, [radius, radius])
}

fn ellipse(center: CurvePoint, radii: CurvePoint) -> CurveShape<CurvePoint> {
    let start = [center[0] + radii[0], center[1]];
    let mut builder = CurveShapeBuilder::new().move_to(start).expect("move_to");

    for index in 0..4 {
        builder = builder
            .arc_to(EllipticArc {
                center,
                radii,
                rotation: 0.0,
                start_angle: index as f32 * FRAC_PI_2,
                sweep_angle: FRAC_PI_2,
            })
            .expect("arc_to");
    }

    builder.build().expect("ellipse")
}

fn blob(
    p0: CurvePoint,
    p1: CurvePoint,
    p2: CurvePoint,
    p3: CurvePoint,
    p4: CurvePoint,
    p5: CurvePoint,
) -> CurveShape<CurvePoint> {
    CurveShapeBuilder::new()
        .move_to(p0)
        .expect("move_to")
        .cubic_to(p1, p2, p3)
        .expect("cubic_to")
        .cubic_to(p4, p5, p0)
        .expect("cubic_to")
        .build()
        .expect("blob")
}

fn capsule(
    left_center: CurvePoint,
    right_center: CurvePoint,
    radius: f32,
) -> CurveShape<CurvePoint> {
    let start = [right_center[0], right_center[1] - radius];
    let end = [left_center[0], left_center[1] + radius];

    CurveShapeBuilder::new()
        .move_to(start)
        .expect("move_to")
        .arc_to(EllipticArc {
            center: right_center,
            radii: [radius, radius],
            rotation: 0.0,
            start_angle: -FRAC_PI_2,
            sweep_angle: PI,
        })
        .expect("arc_to")
        .line_to(end)
        .expect("line_to")
        .arc_to(EllipticArc {
            center: left_center,
            radii: [radius, radius],
            rotation: 0.0,
            start_angle: FRAC_PI_2,
            sweep_angle: PI,
        })
        .expect("arc_to")
        .line_to(start)
        .expect("line_to")
        .build()
        .expect("capsule")
}

fn scaled_cubic_solo(scale: f32) -> CurveShape<CurvePoint> {
    CurveShapeBuilder::new()
        .move_to(scaled([-1.0, 0.0], scale))
        .expect("move_to")
        .cubic_to(
            scaled([-1.0, -0.5], scale),
            scaled([-0.5, -1.0], scale),
            scaled([0.0, -1.0], scale),
        )
        .expect("cubic_to")
        .cubic_to(
            scaled([0.5, -1.0], scale),
            scaled([1.0, -0.5], scale),
            scaled([1.0, 0.0], scale),
        )
        .expect("cubic_to")
        .cubic_to(
            scaled([1.0, 0.5], scale),
            scaled([0.5, 1.0], scale),
            scaled([0.0, 1.0], scale),
        )
        .expect("cubic_to")
        .cubic_to(
            scaled([-0.5, 1.0], scale),
            scaled([-1.0, 0.5], scale),
            scaled([-1.0, 0.0], scale),
        )
        .expect("cubic_to")
        .build()
        .expect("cubic solo")
}

fn scaled(point: CurvePoint, scale: f32) -> CurvePoint {
    [point[0] * scale, point[1] * scale]
}
