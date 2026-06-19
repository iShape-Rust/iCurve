use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub struct CubicSegment<T: FloatNumber> {
    pub control_points: [FloatPoint<T>; 4],
}
