use crate::int::base::spline::IntBaseSpline;
use crate::int::bezier::spline_cubic::IntCubicSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::math::point::IntPoint;

pub trait IntBezierCurveSpline: IntBaseSpline {
    fn anchors(&self) -> &[IntPoint];

    #[inline(always)]
    fn start(&self) -> IntPoint {
        *self.anchors().first().unwrap()
    }

    #[inline]
    fn end(&self) -> IntPoint {
        *self.anchors().last().unwrap()
    }
}

impl IntBezierCurveSpline for IntLineSpline {
    #[inline(always)]
    fn anchors(&self) -> &[IntPoint] {
        &self.anchors
    }
}

impl IntBezierCurveSpline for IntSquareSpline {
    #[inline(always)]
    fn anchors(&self) -> &[IntPoint] {
        &self.anchors
    }
}

impl IntBezierCurveSpline for IntCubicSpline {
    #[inline(always)]
    fn anchors(&self) -> &[IntPoint] {
        &self.anchors
    }
}