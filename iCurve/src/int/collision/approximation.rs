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
use crate::int::math::x_segment::XSegment;

pub trait SplineApproximation: IntBaseSpline + Sized {
    fn to_segment(&self) -> XSegment;

    fn to_convex_line(&self) -> FourVec<IntPoint>;

    fn to_convex(&self) -> FourVec<IntPoint>;



    #[inline(always)]
    fn into_collider(self, space: &Space) -> Collider {
        let boundary = self.boundary();
        let size_level = boundary.size_level();
        let approximation = if size_level >= space.convex_level {
            None
        } else if size_level < space.line_level {
            Some(self.to_convex_line())
        } else {
            Some(self.to_convex())
        };

        Collider {
            spline: self.into_spline(),
            boundary,
            approximation,
            size_level,
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
