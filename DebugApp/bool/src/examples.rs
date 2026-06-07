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
        example_0(),
        example_1(),
        example_2(),
        example_3(),
        example_4(),
        example_5(),
        example_6(),
        example_7(),
        example_8(),
    ]
}

fn example_0() -> BoolExample {
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

fn example_1() -> BoolExample {
    BoolExample {
        name: "circle x rect",
        subject: vec![circle([-65.0, 0.0], 150.0)],
        clip: vec![polygon(&[
            [-10.0, -155.0],
            [205.0, -80.0],
            [145.0, 150.0],
            [-85.0, 120.0],
        ])],
    }
}

fn example_2() -> BoolExample {
    let scale = 200.0;
    let subject = CurveShapeBuilder::new()
        .move_to(scaled([-1.0, 0.0], scale))
        .expect("move_to")
        .line_to(scaled([1.0, 0.0], scale))
        .expect("line_to")
        .quad_to(scaled([0.0, 1.0], scale), scaled([-1.0, 0.0], scale))
        .expect("quad_to")
        .build()
        .expect("subject");

    let clip = CurveShapeBuilder::new()
        .move_to(scaled([-0.5, 0.5], scale))
        .expect("move_to")
        .line_to(scaled([0.5, 0.5], scale))
        .expect("line_to")
        .quad_to(scaled([0.0, 2.0], scale), scaled([-0.5, 0.5], scale))
        .expect("quad_to")
        .build()
        .expect("clip");

    BoolExample {
        name: "quad x quad 0",
        subject: vec![subject],
        clip: vec![clip],
    }
}

fn example_3() -> BoolExample {
    let scale = 200.0;
    let subject = CurveShapeBuilder::new()
        .move_to(scaled([-1.0, 0.0], scale))
        .expect("move_to")
        .line_to(scaled([1.0, 0.0], scale))
        .expect("line_to")
        .quad_to(scaled([0.0, 1.0], scale), scaled([-1.0, 0.0], scale))
        .expect("quad_to")
        .build()
        .expect("subject");

    let clip = CurveShapeBuilder::new()
        .move_to(scaled([-0.5, 0.0], scale))
        .expect("move_to")
        .line_to(scaled([0.5, 0.0], scale))
        .expect("line_to")
        .quad_to(scaled([0.0, 2.0], scale), scaled([-0.5, 0.0], scale))
        .expect("quad_to")
        .build()
        .expect("clip");

    BoolExample {
        name: "quad x quad 1",
        subject: vec![subject],
        clip: vec![clip],
    }
}

fn example_4() -> BoolExample {
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

fn example_5() -> BoolExample {
    BoolExample {
        name: "two subjects",
        subject: vec![circle([-135.0, -35.0], 105.0), circle([45.0, 35.0], 105.0)],
        clip: vec![polygon(&[
            [-85.0, -160.0],
            [225.0, -110.0],
            [160.0, 150.0],
            [-160.0, 135.0],
        ])],
    }
}

fn example_6() -> BoolExample {
    BoolExample {
        name: "arc capsule",
        subject: vec![capsule([-215.0, -95.0], [135.0, 95.0], 82.0)],
        clip: vec![circle([45.0, 0.0], 135.0)],
    }
}

fn example_7() -> BoolExample {
    let scale = 150.0;
    let subject = CurveShapeBuilder::new()
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
        .expect("cubic solo");

    BoolExample {
        name: "cubic solo",
        subject: vec![subject],
        clip: Vec::new(),
    }
}

fn example_8() -> BoolExample {
    let scale = 200.0;
    let cubic = [
        scaled([1.0, 0.0], scale),
        scaled([0.5, 1.0], scale),
        scaled([-0.5, 1.0], scale),
        scaled([-1.0, 0.0], scale),
    ];
    let clip_cubic = cubic_range(cubic, 0.2, 0.8);

    let subject = CurveShapeBuilder::new()
        .move_to(scaled([-1.0, 0.0], scale))
        .expect("move_to")
        .line_to(cubic[0])
        .expect("line_to")
        .cubic_to(cubic[1], cubic[2], cubic[3])
        .expect("cubic_to")
        .build()
        .expect("subject");

    let clip = CurveShapeBuilder::new()
        .move_to(clip_cubic[0])
        .expect("move_to")
        .cubic_to(clip_cubic[1], clip_cubic[2], clip_cubic[3])
        .expect("cubic_to")
        .line_to(clip_cubic[0])
        .expect("line_to")
        .build()
        .expect("clip");

    BoolExample {
        name: "cubic shared range",
        subject: vec![subject],
        clip: vec![clip],
    }
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

    builder
        .close_with_line()
        .expect("close")
        .build()
        .expect("ellipse")
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

fn scaled(point: CurvePoint, scale: f32) -> CurvePoint {
    [point[0] * scale, point[1] * scale]
}

fn cubic_range(cubic: [CurvePoint; 4], t0: f32, t1: f32) -> [CurvePoint; 4] {
    debug_assert!(0.0 <= t0 && t0 < t1 && t1 <= 1.0);

    let [left, _] = split_cubic_at(cubic, t1);
    let local_t = t0 / t1;
    let [_, range] = split_cubic_at(left, local_t);
    range
}

fn split_cubic_at(cubic: [CurvePoint; 4], t: f32) -> [[CurvePoint; 4]; 2] {
    let p01 = line_point(cubic[0], cubic[1], t);
    let p12 = line_point(cubic[1], cubic[2], t);
    let p23 = line_point(cubic[2], cubic[3], t);
    let p012 = line_point(p01, p12, t);
    let p123 = line_point(p12, p23, t);
    let p0123 = line_point(p012, p123, t);

    [[cubic[0], p01, p012, p0123], [p0123, p123, p23, cubic[3]]]
}

fn line_point(a: CurvePoint, b: CurvePoint, t: f32) -> CurvePoint {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
