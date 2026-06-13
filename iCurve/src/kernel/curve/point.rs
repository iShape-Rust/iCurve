use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::param::SegmentParam;
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

pub trait PointAt<T: FloatNumber> {
    fn point_at(&self, param: SegmentParam<T>) -> FloatPoint<T>;
}

pub(crate) trait InnerPointAt<T: FloatNumber> {
    fn inner_point_at(&self, t: T) -> FloatPoint<T>;
}

// --- InnerPointAt

impl<T: FloatNumber> InnerPointAt<T> for [FloatPoint<T>; 2] {
    #[inline]
    fn inner_point_at(&self, t: T) -> FloatPoint<T> {
        let [p0, p1] = *self;
        p0 + (p1 - p0) * t
    }
}

impl<T: FloatNumber> InnerPointAt<T> for [FloatPoint<T>; 3] {
    #[inline]
    fn inner_point_at(&self, t: T) -> FloatPoint<T> {
        let [p0, p1, p2] = *self;
        let i = T::ONE - t;
        let a = i * i;
        let b = T::TWO * t * i;
        let c = t * t;
        p0 * a + p1 * b + p2 * c
    }
}

impl<T: FloatNumber> InnerPointAt<T> for [FloatPoint<T>; 4] {
    #[inline]
    fn inner_point_at(&self, t: T) -> FloatPoint<T> {
        let [p0, p1, p2, p3] = *self;
        let i = T::ONE - t;
        let a = i * i * i;
        let b = T::THREE * t * i * i;
        let c = T::THREE * t * t * i;
        let d = t * t * t;
        p0 * a + p1 * b + p2 * c + p3 * d
    }
}

// --- PointAt

impl<T: FloatNumber> PointAt<T> for [FloatPoint<T>; 2] {
    #[inline]
    fn point_at(&self, param: SegmentParam<T>) -> FloatPoint<T> {
        match param {
            SegmentParam::Start => self[0],
            SegmentParam::End => self[1],
            SegmentParam::Inner(t) => self.inner_point_at(t),
        }
    }
}

impl<T: FloatNumber> PointAt<T> for [FloatPoint<T>; 3] {
    #[inline]
    fn point_at(&self, param: SegmentParam<T>) -> FloatPoint<T> {
        match param {
            SegmentParam::Start => self[0],
            SegmentParam::End => self[2],
            SegmentParam::Inner(t) => self.inner_point_at(t),
        }
    }
}

impl<T: FloatNumber> PointAt<T> for [FloatPoint<T>; 4] {
    #[inline]
    fn point_at(&self, param: SegmentParam<T>) -> FloatPoint<T> {
        match param {
            SegmentParam::Start => self[0],
            SegmentParam::End => self[3],
            SegmentParam::Inner(t) => self.inner_point_at(t),
        }
    }
}

impl<T: FloatNumber> PointAt<T> for LineSegment<T> {
    #[inline]
    fn point_at(&self, param: SegmentParam<T>) -> FloatPoint<T> {
        self.control_points.point_at(param)
    }
}

impl<T: FloatNumber> PointAt<T> for QuadSegment<T> {
    #[inline]
    fn point_at(&self, param: SegmentParam<T>) -> FloatPoint<T> {
        self.control_points.point_at(param)
    }
}

impl<T: FloatNumber> PointAt<T> for CubicSegment<T> {
    #[inline]
    fn point_at(&self, param: SegmentParam<T>) -> FloatPoint<T> {
        self.control_points.point_at(param)
    }
}