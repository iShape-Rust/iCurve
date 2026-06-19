use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonotoneDecompositionDirection {
    X,
    Y,
}

impl MonotoneDecompositionDirection {
    #[inline]
    pub(crate) fn with_rect<T: FloatNumber>(rect: FloatRect<T>) -> Self {
        if rect.width() >= rect.height() {
            Self::X
        } else {
            Self::Y
        }
    }
}

pub trait DecompositeIntoMonotone {
    type Output;
    fn decomposite_into_monotone(&self, direction: MonotoneDecompositionDirection) -> Self::Output;
}
