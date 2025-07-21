use crate::int::bezier::anchor::IntBezierAnchor;
use crate::int::bezier::length::IntSplineLength;
use crate::int::bezier::point::IntSplinePoints;
use crate::int::bezier::short::{IntShort, IntSplineShorts};
use crate::int::bezier::spline_cube::IntCubeSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub(crate) enum IntBezierSpline {
    Line(IntLineSpline),
    Square(IntSquareSpline),
    Cube(IntCubeSpline),
}

impl IntBezierSpline {
    #[inline]
    pub(super) fn new(a: &IntBezierAnchor, b: &IntBezierAnchor) -> Self {
        match (a.handle_out_point(), b.handle_in_point()) {
            (Some(am), Some(bm)) => IntBezierSpline::Cube(IntCubeSpline {
                anchors: [a.point, am, bm, b.point],
            }),
            (Some(m), None) => IntBezierSpline::Square(IntSquareSpline {
                anchors: [a.point, m, b.point],
            }),
            (None, Some(m)) => IntBezierSpline::Square(IntSquareSpline {
                anchors: [a.point, m, b.point],
            }),
            (None, None) => IntBezierSpline::Line(IntLineSpline {
                anchors: [a.point, b.point],
            }),
        }
    }

    #[inline]
    pub fn approximate_points(&self, min_cos: u32, min_len: u32) -> Vec<IntPoint> {
        match self {
            IntBezierSpline::Line(s) => s.approximate_points(min_cos, min_len),
            IntBezierSpline::Square(s) => s.approximate_points(min_cos, min_len),
            IntBezierSpline::Cube(s) => s.approximate_points(min_cos, min_len),
        }
    }

    #[inline]
    pub fn shorts(&self, min_cos: u32, min_len: u32) -> Vec<IntShort> {
        match self {
            IntBezierSpline::Line(s) => s.approximate(min_cos, min_len),
            IntBezierSpline::Square(s) => s.approximate(min_cos, min_len),
            IntBezierSpline::Cube(s) => s.approximate(min_cos, min_len),
        }
    }

    #[inline]
    pub fn avg_length(&self, min_cos: u32, min_len: u32) -> u128 {
        match self {
            IntBezierSpline::Line(s) => s.avg_length(min_cos, min_len),
            IntBezierSpline::Square(s) => s.avg_length(min_cos, min_len),
            IntBezierSpline::Cube(s) => s.avg_length(min_cos, min_len),
        }
    }
}

pub struct SplitPosition {
    pub power: u32,
    pub value: u64,
}

pub trait IntBezierSplineApi: Sized {
    fn start(&self) -> IntPoint;
    fn start_dir(&self) -> IntPoint;
    fn end_dir(&self) -> IntPoint;
    fn end(&self) -> IntPoint;
    fn point_at(&self, position: &SplitPosition) -> IntPoint;
    fn boundary(&self) -> IntRect;
    fn anchors(&self) -> &[IntPoint];
    fn bisect(&self) -> (Self, Self);
}
