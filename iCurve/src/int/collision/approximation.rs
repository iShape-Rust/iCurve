use crate::data::four_vec::FourVec;
use crate::int::base::spline::IntBaseSpline;
use crate::int::bezier::curve::IntBezierCurveSpline;
use crate::int::bezier::spline_cubic::IntCubicSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::circle::arc::IntArc;
use crate::int::collision::collider::Collider;
use crate::int::collision::fit::{FIT_32_ATOM, FIT_32_MAX};
use crate::int::collision::four_convex_path::FourConvexPath;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;
use crate::int::math::x_segment::XSegment;

pub trait SplineApproximation: IntBaseSpline + Sized {

    fn to_segment(&self) -> XSegment;

    fn to_convex_line(&self) -> FourVec<IntPoint>;

    fn to_convex(&self) -> FourVec<IntPoint>;

    #[inline(always)]
    fn convex_approximation(&self, boundary: &IntRect) -> Option<FourVec<IntPoint>> {
        match boundary.max_log_size() {
            0..FIT_32_ATOM => Some(self.to_convex_line()),
            FIT_32_ATOM..FIT_32_MAX => Some(self.to_convex()),
            _ => None,
        }
    }

    #[inline(always)]
    fn into_collider(self) -> Collider {
        let boundary = self.boundary();
        let approximation = self.convex_approximation(&boundary);
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
