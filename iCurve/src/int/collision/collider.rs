use crate::int::bezier::spline::IntBezierSplineApi;
use crate::int::bezier::spline_cube::IntCubeSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::collision::spline::Spline;
use crate::int::math::rect::IntRect;

pub trait SplineCollider {
    fn boundary(&self) -> IntRect;
}

pub struct Collider {
    pub(crate) spline: Spline,
    pub(crate) boundary: IntRect,
}

impl Collider {
    #[inline]
    pub(crate) fn new(spline: Spline) -> Self {
        Self {
            boundary: spline.boundary(),
            spline,
        }
    }

    #[inline]
    pub(crate) fn new_line(spline: IntLineSpline) -> Self {
        Self {
            boundary: spline.boundary(),
            spline: Spline::Line(spline)
        }
    }

    #[inline]
    pub(crate) fn new_square(spline: IntSquareSpline) -> Self {
        Self {
            boundary: spline.boundary(),
            spline: Spline::Square(spline)
        }
    }

    #[inline]
    pub(crate) fn new_cube(spline: IntCubeSpline) -> Self {
        Self {
            boundary: spline.boundary(),
            spline: Spline::Cube(spline)
        }
    }
}