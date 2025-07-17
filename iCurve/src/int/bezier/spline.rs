use alloc::vec::Vec;
use crate::int::bezier::anchor::IntBezierAnchor;
use crate::int::bezier::length::IntSplineLength;
use crate::int::bezier::point::IntSplinePoints;
use crate::int::bezier::short::{IntShort, IntSplineShorts};
use crate::int::bezier::spline_cube::IntCubeSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub(crate) enum IntSpline {
    Line(IntLineSpline),
    Square(IntSquareSpline),
    Cube(IntCubeSpline),
}

impl IntSpline {
    #[inline]
    pub(super) fn new(a: &IntBezierAnchor, b: &IntBezierAnchor) -> Self {
        match (a.handle_out_point(), b.handle_in_point()) {
            (Some(am), Some(bm)) => IntSpline::Cube(IntCubeSpline {
                a: a.point,
                am,
                bm,
                b: b.point,
            }),
            (Some(m), None) => IntSpline::Square(IntSquareSpline {
                a: a.point,
                m,
                b: b.point,
            }),
            (None, Some(m)) => IntSpline::Square(IntSquareSpline {
                a: a.point,
                m,
                b: b.point,
            }),
            (None, None) => IntSpline::Line(IntLineSpline {
                a: a.point,
                b: b.point,
            }),
        }
    }

    #[inline]
    pub fn approximate_points(&self, min_cos: u32, min_len: u32) -> Vec<IntPoint> {
        match self {
            IntSpline::Line(s) => s.approximate_points(min_cos, min_len),
            IntSpline::Square(s) => s.approximate_points(min_cos, min_len),
            IntSpline::Cube(s) => s.approximate_points(min_cos, min_len),
        }
    }

    #[inline]
    pub fn shorts(&self, min_cos: u32, min_len: u32) -> Vec<IntShort> {
        match self {
            IntSpline::Line(s) => s.approximate(min_cos, min_len),
            IntSpline::Square(s) => s.approximate(min_cos, min_len),
            IntSpline::Cube(s) => s.approximate(min_cos, min_len),
        }
    }

    #[inline]
    pub fn avg_length(&self, min_cos: u32, min_len: u32) -> u128 {
        match self {
            IntSpline::Line(s) => s.avg_length(min_cos, min_len),
            IntSpline::Square(s) => s.avg_length(min_cos, min_len),
            IntSpline::Cube(s) => s.avg_length(min_cos, min_len),
        }
    }
}

pub trait IntCADSpline {
    fn start(&self) -> IntPoint;
    fn start_dir(&self) -> IntPoint;
    fn end_dir(&self) -> IntPoint;
    fn end(&self) -> IntPoint;
    fn split_at(&self, step: usize, split_factor: u32) -> IntPoint;
    fn boundary(&self) -> IntRect;
}