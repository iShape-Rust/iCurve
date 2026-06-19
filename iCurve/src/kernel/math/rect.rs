use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

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
