use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::line::FloatLineSegment;
use crate::kernel::float::curve::point_at::InnerPointAt;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use crate::kernel::float::curve::segment::FloatSegment;
use i_overlay::i_float::float::number::FloatNumber;

pub trait FloatSplitAt<T: FloatNumber> {
    type Output;

    fn split_at(&self, t: T) -> Self::Output;
    fn split_at_left(&self, t: T) -> Self;
    fn split_at_right(&self, t: T) -> Self;
}

impl<T: FloatNumber> FloatSplitAt<T> for FloatLineSegment<T> {
    type Output = [Self; 2];

    #[inline]
    fn split_at(&self, t: T) -> Self::Output {
        let m = self.control_points.inner_point_at(t);
        let [p0, p1] = self.control_points;
        [
            FloatLineSegment {
                control_points: [p0, m],
            },
            FloatLineSegment {
                control_points: [m, p1],
            },
        ]
    }

    #[inline]
    fn split_at_left(&self, t: T) -> Self {
        let m = self.control_points.inner_point_at(t);
        FloatLineSegment {
            control_points: [self.control_points[0], m],
        }
    }

    #[inline]
    fn split_at_right(&self, t: T) -> Self {
        let m = self.control_points.inner_point_at(t);
        FloatLineSegment {
            control_points: [m, self.control_points[1]],
        }
    }
}

impl<T: FloatNumber> FloatSplitAt<T> for FloatQuadSegment<T> {
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

impl<T: FloatNumber> FloatSplitAt<T> for FloatCubicSegment<T> {
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

impl<T: FloatNumber> FloatSplitAt<T> for FloatSegment<T> {
    type Output = [Self; 2];

    fn split_at(&self, t: T) -> Self::Output {
        match self {
            FloatSegment::Line(line) => {
                let [lt, rt] = line.split_at(t);
                [FloatSegment::Line(lt), FloatSegment::Line(rt)]
            }
            FloatSegment::Quad(quad) => {
                let [lt, rt] = quad.split_at(t);
                [FloatSegment::Quad(lt), FloatSegment::Quad(rt)]
            }
            FloatSegment::Cubic(cubic) => {
                let [lt, rt] = cubic.split_at(t);
                [FloatSegment::Cubic(lt), FloatSegment::Cubic(rt)]
            }
        }
    }

    fn split_at_left(&self, t: T) -> Self {
        match self {
            FloatSegment::Line(line) => FloatSegment::Line(line.split_at_left(t)),
            FloatSegment::Quad(quad) => FloatSegment::Quad(quad.split_at_left(t)),
            FloatSegment::Cubic(cubic) => FloatSegment::Cubic(cubic.split_at_left(t)),
        }
    }

    fn split_at_right(&self, t: T) -> Self {
        match self {
            FloatSegment::Line(line) => FloatSegment::Line(line.split_at_right(t)),
            FloatSegment::Quad(quad) => FloatSegment::Quad(quad.split_at_right(t)),
            FloatSegment::Cubic(cubic) => FloatSegment::Cubic(cubic.split_at_right(t)),
        }
    }
}
