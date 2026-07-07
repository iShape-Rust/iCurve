use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::line::FloatLineSegment;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use crate::kernel::float::curve::segment::FloatSegment;
use i_overlay::i_float::adapter::FloatPointAdapter;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::rect::IntRect;

pub trait ToRect<T: FloatNumber> {
    fn to_rect(&self) -> FloatRect<T>;
}

impl<T: FloatNumber> ToRect<T> for FloatLineSegment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        let mut rect = FloatRect::with_point(self.control_points[0]);
        rect.unsafe_add_point(&self.control_points[1]);
        rect
    }
}

impl<T: FloatNumber> ToRect<T> for FloatQuadSegment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        let mut rect = FloatRect::with_point(self.control_points[0]);
        rect.unsafe_add_point(&self.control_points[1]);
        rect.unsafe_add_point(&self.control_points[2]);
        rect
    }
}

impl<T: FloatNumber> ToRect<T> for FloatCubicSegment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        let mut rect = FloatRect::with_point(self.control_points[0]);
        rect.unsafe_add_point(&self.control_points[1]);
        rect.unsafe_add_point(&self.control_points[2]);
        rect.unsafe_add_point(&self.control_points[3]);
        rect
    }
}

impl<T: FloatNumber> ToRect<T> for FloatSegment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        match self {
            FloatSegment::Line(line) => line.to_rect(),
            FloatSegment::Quad(quad) => quad.to_rect(),
            FloatSegment::Cubic(cubic) => cubic.to_rect(),
        }
    }
}

pub trait ToIntRect<T: FloatNumber, I: IntNumber> {
    fn to_int_rect(&self, adapter: &FloatPointAdapter<FloatPoint<T>, I>) -> IntRect<I>;
}

impl<T: FloatNumber, I: IntNumber> ToIntRect<T, I> for FloatRect<T> {
    #[inline]
    fn to_int_rect(&self, adapter: &FloatPointAdapter<FloatPoint<T>, I>) -> IntRect<I> {
        let min = adapter.float_to_int(&FloatPoint::new(self.min_x, self.min_y));
        let max = adapter.float_to_int(&FloatPoint::new(self.max_x, self.max_y));
        IntRect::with_min_max(min, max)
    }
}
