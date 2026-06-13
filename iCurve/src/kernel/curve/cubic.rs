use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::float::rect::FloatRect;

pub struct CubicSegment<T: FloatNumber> {
    pub control_points: [FloatPoint<T>; 4],
}

impl<T: FloatNumber> CubicSegment<T> {
    #[inline]
    pub fn to_rect(&self) -> FloatRect<T> {
        let mut rect = FloatRect::with_point(self.control_points[0]);
        rect.unsafe_add_point(&self.control_points[1]);
        rect.unsafe_add_point(&self.control_points[2]);
        rect.unsafe_add_point(&self.control_points[3]);
        rect
    }
}
