use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::line::FloatLineSegment;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use i_overlay::i_float::float::number::FloatNumber;

pub trait Reversed {
    fn reversed(&self) -> Self;
}

impl<T: FloatNumber> Reversed for FloatLineSegment<T> {
    #[inline(always)]
    fn reversed(&self) -> Self {
        let [p0, p1] = self.control_points;
        Self {
            control_points: [p1, p0],
        }
    }
}

impl<T: FloatNumber> Reversed for FloatQuadSegment<T> {
    #[inline(always)]
    fn reversed(&self) -> Self {
        let [p0, p1, p2] = self.control_points;
        Self {
            control_points: [p2, p1, p0],
        }
    }
}

impl<T: FloatNumber> Reversed for FloatCubicSegment<T> {
    #[inline(always)]
    fn reversed(&self) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        Self {
            control_points: [p3, p2, p1, p0],
        }
    }
}
