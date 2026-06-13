use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::point::InnerPointAt;
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;

pub trait SplitAt<T: FloatNumber> {
    type Output;

    fn split_at(&self, t: T) -> Self::Output;
    fn split_at_left(&self, t: T) -> Self;
    fn split_at_right(&self, t: T) -> Self;
}

impl<T: FloatNumber> SplitAt<T> for LineSegment<T> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: T) -> Self::Output {
        let m = self.control_points.inner_point_at(t);
        let [p0, p1] = self.control_points;
        [
            LineSegment {
                control_points: [p0, m],
            },
            LineSegment {
                control_points: [m, p1],
            },
        ]
    }

    #[inline]
    fn split_at_left(&self, t: T) -> Self {
        let m = self.control_points.inner_point_at(t);
        LineSegment {
            control_points: [self.control_points[0], m],
        }
    }

    #[inline]
    fn split_at_right(&self, t: T) -> Self {
        let m = self.control_points.inner_point_at(t);
        LineSegment {
            control_points: [m, self.control_points[1]],
        }
    }
}

impl<T: FloatNumber> SplitAt<T> for QuadSegment<T> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: T) -> Self::Output {
        let [p0, p1, p2] = self.control_points;

        let p01 = [p0, p1].inner_point_at(t);
        let p12 = [p1, p2].inner_point_at(t);
        let p012 = [p01, p12].inner_point_at(t);

        [
            Self {
                control_points: [p0, p01, p012],
            },
            Self {
                control_points: [p012, p12, p2],
            },
        ]
    }

    #[inline]
    fn split_at_left(&self, t: T) -> Self {
        let [p0, p1, p2] = self.control_points;

        let p01 = [p0, p1].inner_point_at(t);
        let p12 = [p1, p2].inner_point_at(t);
        let p012 = [p01, p12].inner_point_at(t);

        Self {
            control_points: [p0, p01, p012],
        }
    }

    #[inline]
    fn split_at_right(&self, t: T) -> Self {
        let [p0, p1, p2] = self.control_points;

        let p01 = [p0, p1].inner_point_at(t);
        let p12 = [p1, p2].inner_point_at(t);
        let p012 = [p01, p12].inner_point_at(t);

        Self {
            control_points: [p012, p12, p2],
        }
    }
}

impl<T: FloatNumber> SplitAt<T> for CubicSegment<T> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: T) -> Self::Output {
        let [p0, p1, p2, p3] = self.control_points;
        let p01 = [p0, p1].inner_point_at(t);
        let p12 = [p1, p2].inner_point_at(t);
        let p23 = [p2, p3].inner_point_at(t);
        let p012 = [p01, p12].inner_point_at(t);
        let p123 = [p12, p23].inner_point_at(t);
        let p0123 = [p012, p123].inner_point_at(t);

        [
            Self {
                control_points: [p0, p01, p012, p0123],
            },
            Self {
                control_points: [p0123, p123, p23, p3],
            },
        ]
    }

    #[inline]
    fn split_at_left(&self, t: T) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        let p01 = [p0, p1].inner_point_at(t);
        let p12 = [p1, p2].inner_point_at(t);
        let p23 = [p2, p3].inner_point_at(t);
        let p012 = [p01, p12].inner_point_at(t);
        let p123 = [p12, p23].inner_point_at(t);
        let p0123 = [p012, p123].inner_point_at(t);

        Self {
            control_points: [p0, p01, p012, p0123],
        }
    }

    #[inline]
    fn split_at_right(&self, t: T) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        let p01 = [p0, p1].inner_point_at(t);
        let p12 = [p1, p2].inner_point_at(t);
        let p23 = [p2, p3].inner_point_at(t);
        let p012 = [p01, p12].inner_point_at(t);
        let p123 = [p12, p23].inner_point_at(t);
        let p0123 = [p012, p123].inner_point_at(t);

        Self {
            control_points: [p0123, p123, p23, p3],
        }
    }
}
