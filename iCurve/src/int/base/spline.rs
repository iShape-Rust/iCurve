use crate::int::bezier::spline_cubic::IntCubicSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::circle::arc::IntArc;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub enum IntSpline {
    Arc(IntArc),
    Line(IntLineSpline),
    Square(IntSquareSpline),
    Cubic(IntCubicSpline),
}

pub trait IntBaseSpline {
    fn into_spline(self) -> IntSpline;
    fn boundary(&self) -> IntRect;
}

impl IntBaseSpline for IntLineSpline {
    #[inline(always)]
    fn into_spline(self) -> IntSpline {
        IntSpline::Line(self)
    }

    #[inline(always)]
    fn boundary(&self) -> IntRect {
        IntRect::with_points(&self.anchors)
    }
}

impl IntBaseSpline for IntSquareSpline {
    #[inline(always)]
    fn into_spline(self) -> IntSpline {
        IntSpline::Square(self)
    }

    #[inline(always)]
    fn boundary(&self) -> IntRect {
        IntRect::with_points(&self.anchors)
    }
}

impl IntBaseSpline for IntCubicSpline {
    #[inline(always)]
    fn into_spline(self) -> IntSpline {
        IntSpline::Cubic(self)
    }

    #[inline(always)]
    fn boundary(&self) -> IntRect {
        IntRect::with_points(&self.anchors)
    }
}

impl IntBaseSpline for IntArc {
    #[inline(always)]
    fn into_spline(self) -> IntSpline {
        IntSpline::Arc(self)
    }

    #[inline(always)]
    fn boundary(&self) -> IntRect {
        panic!("Not Implemented")
    }
}
