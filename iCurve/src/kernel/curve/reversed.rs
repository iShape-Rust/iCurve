use i_overlay::i_float::float::number::FloatNumber;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;

pub trait Reversed {
    fn reversed(&self) -> Self;
}

impl<T:FloatNumber> Reversed for LineSegment<T> {
    #[inline(always)]
    fn reversed(&self) -> Self {
        let [p0, p1] = self.control_points;
        Self {
            control_points: [p1, p0],
        }
    }
}

impl<T:FloatNumber> Reversed for QuadSegment<T> {
    #[inline(always)]
    fn reversed(&self) -> Self {
        let [p0, p1, p2] = self.control_points;
        Self {
            control_points: [p2, p1, p0],
        }
    }
}

impl<T:FloatNumber> Reversed for CubicSegment<T> {
    #[inline(always)]
    fn reversed(&self) -> Self {
        let [p0, p1, p2, p3] = self.control_points;
        Self {
            control_points: [p3, p2, p1, p0],
        }
    }
}