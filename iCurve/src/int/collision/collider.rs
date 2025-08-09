use crate::data::four_vec::FourVec;
use crate::int::base::spline::IntSpline;
use crate::int::bezier::spline::IntBezierSplineMath;
use crate::int::collision::approximation::SplineApproximation;
use crate::int::collision::convex::Convexity;
use crate::int::collision::space::Space;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;
use crate::int::math::x_segment::XSegment;

#[derive(Clone)]
pub struct Collider {
    pub spline: IntSpline,
    pub boundary: IntRect,
    pub approximation: Option<FourVec<IntPoint>>,
    pub(crate) size_level: u32,
}

impl Collider {
    #[inline]
    pub(super) fn overlap(&self, other: &Self, space: &Space) -> bool {
        if self.boundary.is_not_overlap(&other.boundary) {
            return false;
        }

        match (&self.approximation, &other.approximation) {
            (Some(c0), Some(c1)) => {
                let max_level = self.size_level.max(other.size_level);
                let margin = (2 + max_level - space.line_level) as u64;
                c0.slice().overlap_with_margin(c1.slice(), margin)
            },
            (_, _) => true,
        }
    }

    #[inline]
    pub(crate) fn split(&self, space: &Space) -> Option<(Collider, Collider)> {
        if self.boundary.size_level() <= space.line_level {
            return None;
        }
        Some(self.bisect(space))
    }

    #[inline]
    fn bisect(&self, space: &Space) -> (Collider, Collider) {
        match &self.spline {
            IntSpline::Arc(_) => panic!("Not implemented"),
            IntSpline::Line(s) => {
                let (s0, s1) = s.bisect();
                (s0.into_collider(space), s1.into_collider(space))
            }
            IntSpline::Square(s) => {
                let (s0, s1) = s.bisect();
                (s0.into_collider(space), s1.into_collider(space))
            }
            IntSpline::Cubic(s) => {
                let (s0, s1) = s.bisect();
                (s0.into_collider(space), s1.into_collider(space))
            }
        }
    }

    #[inline]
    pub(crate) fn to_segment(&self) -> XSegment {
        match &self.spline {
            IntSpline::Arc(_) => panic!("Not implemented"),
            IntSpline::Line(s) => s.to_segment(),
            IntSpline::Square(s) => s.to_segment(),
            IntSpline::Cubic(s) => s.to_segment(),
        }
    }
}
