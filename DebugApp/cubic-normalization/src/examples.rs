use i_overlay::i_shape::int::IntPoint;

pub type Point = IntPoint<i32>;

#[derive(Clone, Copy)]
pub struct CubicExample {
    pub name: &'static str,
    pub points: [Point; 4],
}

pub fn examples() -> Vec<CubicExample> {
    vec![
        CubicExample {
            name: "self intersection",
            points: [
                Point::new(0, 0),
                Point::new(-21, -21),
                Point::new(-21, -14),
                Point::new(-14, -14),
            ],
        },
        CubicExample {
            name: "loop",
            points: [
                Point::new(-120, 0),
                Point::new(160, -180),
                Point::new(-160, -180),
                Point::new(120, 0),
            ],
        },
        CubicExample {
            name: "plain cubic",
            points: [
                Point::new(-140, 0),
                Point::new(-70, 120),
                Point::new(80, 120),
                Point::new(150, 0),
            ],
        },
        CubicExample {
            name: "closed area",
            points: [
                Point::new(0, 0),
                Point::new(120, 180),
                Point::new(-120, 180),
                Point::new(0, 0),
            ],
        },
        CubicExample {
            name: "equal middle",
            points: [
                Point::new(-120, 0),
                Point::new(0, 100),
                Point::new(0, 100),
                Point::new(120, 0),
            ],
        },
        CubicExample {
            name: "collinear",
            points: [
                Point::new(-150, 0),
                Point::new(-50, 0),
                Point::new(50, 0),
                Point::new(150, 0),
            ],
        },
    ]
}
