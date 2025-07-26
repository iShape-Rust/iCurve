use crate::data::four_vec::FourVec;
use crate::int::bezier::spline::IntBezierSplineApi;
use crate::int::bezier::spline_cube::IntCubeSpline;
use crate::int::bezier::spline_line::IntLineSpline;
use crate::int::bezier::spline_square::IntSquareSpline;
use crate::int::circle::arc::IntArc;
use crate::int::collision::collider::Collider;
use crate::int::collision::convex_hull::FourConvexPathExt;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;
use crate::int::math::x_segment::XSegment;

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
            Spline::Arc(spline) => panic!("Not implemented"),
            Spline::Line(spline) => spline.boundary(),
            Spline::Square(spline) => spline.boundary(),
            Spline::Cube(spline) => spline.boundary(),
        }
    }

    #[inline]
    pub fn convex_hull(&self) -> FourVec<IntPoint> {
        match self {
            Spline::Arc(spline) => panic!("Not implemented"),
            Spline::Line(spline) => spline.anchors().to_convex_hull(),
            Spline::Square(spline) => spline.anchors().to_convex_hull(),
            Spline::Cube(spline) => spline.anchors().to_convex_hull(),
        }
    }

    #[inline]
    pub fn as_x_segment(&self) -> XSegment {
        match self {
            Spline::Arc(spline) => panic!("Not implemented"),
            Spline::Line(spline) => XSegment::new(*spline.anchors.first().unwrap(), *spline.anchors.last().unwrap()),
            Spline::Square(spline) => XSegment::new(*spline.anchors.first().unwrap(), *spline.anchors.last().unwrap()),
            Spline::Cube(spline) => XSegment::new(*spline.anchors.first().unwrap(), *spline.anchors.last().unwrap()),
        }
    }

    #[inline]
    pub(crate) fn bisect(&self) -> (Collider, Collider) {
        match self {
            Spline::Arc(spline) => panic!("Not implemented"),
            Spline::Line(spline) => {
                let (s0, s1) = spline.bisect();
                (Collider::new_line(s0), Collider::new_line(s1))
            },
            Spline::Square(spline) => {
                let (s0, s1) = spline.bisect();
                (Collider::new_square(s0), Collider::new_square(s1))
            },
            Spline::Cube(spline) => {
                let (s0, s1) = spline.bisect();
                (Collider::new_cube(s0), Collider::new_cube(s1))
            },
        }
    }
}