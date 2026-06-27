use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub struct FloatCubicSegment<T: FloatNumber> {
    pub control_points: [FloatPoint<T>; 4],
}

impl<T: FloatNumber> Default for FloatCubicSegment<T> {
    #[inline]
    fn default() -> Self {
        Self {
            control_points: [FloatPoint::zero(); 4],
        }
    }
}
