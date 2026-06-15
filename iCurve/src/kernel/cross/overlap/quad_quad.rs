use i_overlay::i_float::float::number::FloatNumber;
use crate::kernel::cross::overlap::find::{CurveOverlap, FindOverlap};
use crate::kernel::curve::quad::QuadSegment;

impl<T: FloatNumber> FindOverlap<T> for QuadSegment<T> {
    fn find_overlap(&self, other: &Self, epsilon: T) -> Option<CurveOverlap<T>> {
        let a0 = self.control_points[0];
        let a2 = self.control_points[2];
        let b0 = other.control_points[0];
        let b2 = other.control_points[2];

        let is_a0 = other.contains(a0, epsilon);
        let is_a2 = other.contains(a2, epsilon);

        None
    }
}