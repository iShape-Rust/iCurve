use debug_ui::egui::Pos2;

#[derive(Clone)]
pub struct CrossExample {
    pub name: &'static str,
    pub curve_a: CrossCurve,
    pub curve_b: CrossCurve,
}

#[derive(Clone)]
pub enum CrossCurve {
    Line([Pos2; 2]),
    Quad([Pos2; 3]),
}

impl CrossCurve {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Line(_) => "line",
            Self::Quad(_) => "quad",
        }
    }
}

pub fn load_examples() -> Vec<CrossExample> {
    vec![
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
        curve_a: CrossCurve::Line([Pos2::new(-180.0, -120.0), Pos2::new(180.0, 120.0)]),
        curve_b: CrossCurve::Line([Pos2::new(-180.0, 120.0), Pos2::new(180.0, -120.0)]),
    }
}

fn line_line_endpoint() -> CrossExample {
    CrossExample {
        name: "line x line: endpoint",
        curve_a: CrossCurve::Line([Pos2::new(-170.0, -80.0), Pos2::new(20.0, 70.0)]),
        curve_b: CrossCurve::Line([Pos2::new(20.0, 70.0), Pos2::new(190.0, -90.0)]),
    }
}

fn line_line_overlap() -> CrossExample {
    CrossExample {
        name: "line x line: overlap",
        curve_a: CrossCurve::Line([Pos2::new(-180.0, 0.0), Pos2::new(90.0, 0.0)]),
        curve_b: CrossCurve::Line([Pos2::new(-40.0, 0.0), Pos2::new(180.0, 0.0)]),
    }
}

fn line_quad_two_points() -> CrossExample {
    CrossExample {
        name: "line x quad: two points",
        curve_a: CrossCurve::Line([Pos2::new(-210.0, 20.0), Pos2::new(210.0, 20.0)]),
        curve_b: CrossCurve::Quad([
            Pos2::new(-170.0, -130.0),
            Pos2::new(0.0, 210.0),
            Pos2::new(170.0, -130.0),
        ]),
    }
}

fn line_quad_tangent() -> CrossExample {
    CrossExample {
        name: "line x quad: tangent",
        curve_a: CrossCurve::Line([Pos2::new(-200.0, 0.0), Pos2::new(200.0, 0.0)]),
        curve_b: CrossCurve::Quad([
            Pos2::new(-100.0, -100.0),
            Pos2::new(0.0, 100.0),
            Pos2::new(100.0, -100.0),
        ]),
    }
}

fn line_quad_no_hit() -> CrossExample {
    CrossExample {
        name: "line x quad: no hit",
        curve_a: CrossCurve::Line([Pos2::new(-210.0, 170.0), Pos2::new(210.0, 170.0)]),
        curve_b: CrossCurve::Quad([
            Pos2::new(-160.0, -120.0),
            Pos2::new(0.0, 105.0),
            Pos2::new(160.0, -120.0),
        ]),
    }
}
