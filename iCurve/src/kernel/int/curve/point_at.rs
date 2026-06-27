use crate::kernel::int::curve::param::SegmentParam;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub trait PointAt<I: IntNumber> {
    fn point_at(&self, param: SegmentParam<I>) -> IntPoint<I>;
}

impl<I: IntNumber> PointAt<I> for [IntPoint<I>; 2] {
    #[inline(always)]
    fn point_at(&self, t: SegmentParam<I>) -> IntPoint<I> {
        let [p0, p1] = *self;
        p0 + t.scale_vector_to_point(p1 - p0)
    }
}

impl<I: IntNumber> PointAt<I> for [IntPoint<I>; 3] {
    #[inline(always)]
    fn point_at(&self, t: SegmentParam<I>) -> IntPoint<I> {
        let [p0, p1, p2] = *self;
        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        [p01, p12].point_at(t)
    }
}

impl<I: IntNumber> PointAt<I> for [IntPoint<I>; 4] {
    #[inline(always)]
    fn point_at(&self, t: SegmentParam<I>) -> IntPoint<I> {
        let [p0, p1, p2, p3] = *self;
        let p01 = [p0, p1].point_at(t);
        let p12 = [p1, p2].point_at(t);
        let p23 = [p2, p3].point_at(t);

        let p012 = [p01, p12].point_at(t);
        let p123 = [p12, p23].point_at(t);

        [p012, p123].point_at(t)
    }
}
