use i_overlay::i_float::float::number::FloatNumber;
use crate::kernel::cross::overlap::find::{CurveOverlap, FindOverlap};
use crate::kernel::curve::quad::QuadSegment;

impl<T: FloatNumber> FindOverlap<T> for QuadSegment<T> {
    fn find_overlap(&self, other: &Self, epsilon: T) -> Option<CurveOverlap<T>> {
        todo!()
    }
}