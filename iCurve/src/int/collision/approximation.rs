use crate::data::four_vec::FourVec;
use crate::int::base::spline::IntBaseSpline;
use crate::int::bezier::curve::IntBezierCurveSpline;
use crate::int::bezier::spline_cubic::IntCubicSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::circle::arc::IntArc;
use crate::int::collision::collider::Collider;
use crate::int::collision::four_convex_path::FourConvexPath;
use crate::int::collision::space::Space;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;
use crate::int::math::x_segment::XSegment;

pub trait SplineApproximation: IntBaseSpline + Sized {
    fn to_segment(&self) -> XSegment;

    fn to_convex_line(&self) -> FourVec<IntPoint>;

    fn to_convex(&self) -> FourVec<IntPoint>;

    #[inline(always)]
    fn convex_approximation(&self, boundary: &IntRect, space: &Space) -> Option<FourVec<IntPoint>> {
        let max_log_size = boundary.max_log_size();
        if max_log_size >= space.convex_level {
            return None;
        }

        let path = if max_log_size < space.line_level {
            self.to_convex_line()
        } else {
            self.to_convex()
        };

        Some(path)
    }

    #[inline(always)]
    fn into_collider(self, space: &Space) -> Collider {
        let boundary = self.boundary();
        let approximation = self.convex_approximation(&boundary, space);

        Collider {
            spline: self.into_spline(),
            boundary,
            approximation,
        }
    }
}

impl SplineApproximation for IntLineSpline {
    #[inline(always)]
    fn to_segment(&self) -> XSegment {
        XSegment::new(self.start(), self.end())
    }

    #[inline(always)]
    fn to_convex_line(&self) -> FourVec<IntPoint> {
        FourVec::line(self.start(), self.end())
    }

    #[inline(always)]
    fn to_convex(&self) -> FourVec<IntPoint> {
        self.anchors().to_four_convex()
    }
}

impl SplineApproximation for IntSquareSpline {
    #[inline(always)]
    fn to_segment(&self) -> XSegment {
        XSegment::new(self.start(), self.end())
    }

    #[inline(always)]
    fn to_convex_line(&self) -> FourVec<IntPoint> {
        FourVec::line(self.start(), self.end())
    }

    #[inline(always)]
    fn to_convex(&self) -> FourVec<IntPoint> {
        self.anchors().to_four_convex()
    }
}

impl SplineApproximation for IntCubicSpline {
    #[inline(always)]
    fn to_segment(&self) -> XSegment {
        XSegment::new(self.start(), self.end())
    }

    #[inline(always)]
    fn to_convex_line(&self) -> FourVec<IntPoint> {
        FourVec::line(self.start(), self.end())
    }

    #[inline(always)]
    fn to_convex(&self) -> FourVec<IntPoint> {
        self.anchors().to_four_convex()
    }
}

impl SplineApproximation for IntArc {
    #[inline(always)]
    fn to_segment(&self) -> XSegment {
        panic!("Not implemented")
    }

    #[inline(always)]
    fn to_convex_line(&self) -> FourVec<IntPoint> {
        panic!("Not implemented")
    }

    #[inline(always)]
    fn to_convex(&self) -> FourVec<IntPoint> {
        panic!("Not implemented")
    }
}

impl FourVec<IntPoint> {
    #[inline(always)]
    fn line(a: IntPoint, b: IntPoint) -> Self {
        FourVec {
            buffer: [a, b, IntPoint::zero(), IntPoint::zero()],
            len: 2,
        }
    }
}
