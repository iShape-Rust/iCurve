use crate::data::four_vec::FourVec;
use crate::int::base::spline::IntSpline;
use crate::int::bezier::spline::IntBezierSplineMath;
use crate::int::collision::approximation::SplineApproximation;
use crate::int::collision::convexity::Convexity;
use crate::int::collision::fit::FIT_32_ATOM;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;
use crate::int::math::x_segment::XSegment;

#[derive(Clone)]
pub struct Collider {
    pub spline: IntSpline,
    pub boundary: IntRect,
    pub approximation: Option<FourVec<IntPoint>>,
}

impl Collider {
    #[inline]
    pub(super) fn overlap(&self, other: &Self) -> bool {
        if self.boundary.is_not_overlap(&other.boundary) {
            return false;
        }

        match (&self.approximation, &other.approximation) {
            (Some(c0), Some(c1)) => c0.slice().overlaps_with_unit_margin(c1.slice()),
            (_, _) => true,
        }
    }

    #[inline]
    pub(crate) fn split(&self) -> Option<(Collider, Collider)> {
        if self.boundary.max_log_size() < FIT_32_ATOM {
            return None;
        }
        Some(self.bisect())
    }

    #[inline]
    fn bisect(&self) -> (Collider, Collider) {
        match &self.spline {
            IntSpline::Arc(_) => panic!("Not implemented"),
            IntSpline::Line(s) => {
                let (s0, s1) = s.bisect();
                (s0.into_collider(), s1.into_collider())
            }
            IntSpline::Square(s) => {
                let (s0, s1) = s.bisect();
                (s0.into_collider(), s1.into_collider())
            }
            IntSpline::Cubic(s) => {
                let (s0, s1) = s.bisect();
                (s0.into_collider(), s1.into_collider())
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
