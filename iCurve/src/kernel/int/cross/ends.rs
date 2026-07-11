use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

pub(super) trait ShareEnds {
    fn is_share_only_one_end(&self, other: &Self) -> bool;
}

impl<I: IntNumber> ShareEnds for [IntPoint<I>; 2] {
    #[inline]
    fn is_share_only_one_end(&self, other: &Self) -> bool {
        self[0] == other[0] && self[1] != other[1]
            || self[0] == other[1] && self[1] != other[0]
            || self[1] == other[0] && self[0] != other[1]
            || self[1] == other[1] && self[0] != other[0]
    }
}
