use crate::int::bezier::spline::IntBezierSplineApi;
use crate::int::bezier::spline_cube::IntCubeSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::circle::arc::IntArc;
use crate::int::math::rect::IntRect;

pub enum Spline {
    Arc(IntArc),
    Line(IntLineSpline),
    Square(IntSquareSpline),
    Cube(IntCubeSpline),
}

impl Spline {

    #[inline]
    pub fn boundary(&self) -> IntRect {
        match self {
            Spline::Arc(spline) => IntRect::empty(),
            Spline::Line(spline) => spline.boundary(),
            Spline::Square(spline) => spline.boundary(),
            Spline::Cube(spline) => spline.boundary(),
        }
    }
}