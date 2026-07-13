use i_overlay::i_shape::int::IntPoint;

pub type Point = IntPoint<i32>;

#[derive(Clone)]
pub struct CrossExample {
    pub name: &'static str,
    pub curve_a: CrossCurve,
    pub curve_b: CrossCurve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossCurve {
    Line([Point; 2]),
    Quad([Point; 3]),
    Cubic([Point; 4]),
}

impl CrossCurve {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Line(_) => "line",
            Self::Quad(_) => "quad",
            Self::Cubic(_) => "cubic",
        }
    }
}

pub fn load_examples() -> Vec<CrossExample> {
    vec![
        cubic_cubic_cross(),
        line_line_cross(),
        line_line_endpoint(),
        line_line_overlap(),
        line_quad_two_points(),
        line_quad_tangent(),
        line_quad_no_hit(),
    ]
}

fn line_line_cross() -> CrossExample {
    CrossExample {
        name: "line x line: cross",
        curve_a: CrossCurve::Line([Point::new(-64, -64), Point::new(64, 64)]),
        curve_b: CrossCurve::Line([Point::new(-64, 64), Point::new(64, -64)]),
    }
}

fn line_line_endpoint() -> CrossExample {
    CrossExample {
        name: "line x line: endpoint",
        curve_a: CrossCurve::Line([Point::new(-170, -80), Point::new(20, 70)]),
        curve_b: CrossCurve::Line([Point::new(20, 70), Point::new(190, -90)]),
    }
}

fn line_line_overlap() -> CrossExample {
    CrossExample {
        name: "line x line: overlap",
        curve_a: CrossCurve::Line([Point::new(-180, 0), Point::new(90, 0)]),
        curve_b: CrossCurve::Line([Point::new(-40, 0), Point::new(180, 0)]),
    }
}

fn line_quad_two_points() -> CrossExample {
    CrossExample {
        name: "line x quad: two points",
        curve_a: CrossCurve::Line([Point::new(-210, 20), Point::new(210, 20)]),
        curve_b: CrossCurve::Quad([
            Point::new(-170, -130),
            Point::new(0, 210),
            Point::new(170, -130),
        ]),
    }
}

fn line_quad_tangent() -> CrossExample {
    CrossExample {
        name: "line x quad: tangent",
        curve_a: CrossCurve::Line([Point::new(-200, 0), Point::new(200, 0)]),
        curve_b: CrossCurve::Quad([
            Point::new(-100, -100),
            Point::new(0, 100),
            Point::new(100, -100),
        ]),
    }
}

fn line_quad_no_hit() -> CrossExample {
    CrossExample {
        name: "line x quad: no hit",
        curve_a: CrossCurve::Line([Point::new(-210, 170), Point::new(210, 170)]),
        curve_b: CrossCurve::Quad([
            Point::new(-160, -120),
            Point::new(0, 105),
            Point::new(160, -120),
        ]),
    }
}

fn cubic_cubic_cross() -> CrossExample {
    CrossExample {
        name: "cubic x cubic: cross",
        curve_a: CrossCurve::Cubic([
            Point::new(100, 100),
            Point::new(100, 400),
            Point::new(600, 900),
            Point::new(1000, 900),
        ]),
        curve_b: CrossCurve::Cubic([
            Point::new(100, 900),
            Point::new(100, 500),
            Point::new(600, 0),
            Point::new(1000, 0),
        ]),
    }
}
