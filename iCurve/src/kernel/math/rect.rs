use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::rect::IntRect;

pub trait ToRect<T: FloatNumber> {
    fn to_rect(&self) -> FloatRect<T>;
}

impl<T: FloatNumber> ToRect<T> for LineSegment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        let mut rect = FloatRect::with_point(self.control_points[0]);
        rect.unsafe_add_point(&self.control_points[1]);
        rect
    }
}

impl<T: FloatNumber> ToRect<T> for QuadSegment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        let mut rect = FloatRect::with_point(self.control_points[0]);
        rect.unsafe_add_point(&self.control_points[1]);
        rect.unsafe_add_point(&self.control_points[2]);
        rect
    }
}

impl<T: FloatNumber> ToRect<T> for CubicSegment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        let mut rect = FloatRect::with_point(self.control_points[0]);
        rect.unsafe_add_point(&self.control_points[1]);
        rect.unsafe_add_point(&self.control_points[2]);
        rect.unsafe_add_point(&self.control_points[3]);
        rect
    }
}

impl<T: FloatNumber> ToRect<T> for Segment<T> {
    #[inline]
    fn to_rect(&self) -> FloatRect<T> {
        match self {
            Segment::Line(line) => line.to_rect(),
            Segment::Quad(quad) => quad.to_rect(),
            Segment::Cubic(cubic) => cubic.to_rect(),
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
