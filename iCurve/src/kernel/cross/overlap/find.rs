
use crate::kernel::cross::contact::ContactPoint;
use i_overlay::i_float::float::number::FloatNumber;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CurveOverlap<T: FloatNumber> {
    pub start: ContactPoint<T>,
    pub end: ContactPoint<T>,
}

pub(crate) trait FindOverlap<T: FloatNumber> {
    fn find_overlap(&self, other: &Self, epsilon: T) -> Option<CurveOverlap<T>>;
}