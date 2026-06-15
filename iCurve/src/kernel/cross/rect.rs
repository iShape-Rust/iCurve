use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

pub(crate) trait FloatRectExt<T: FloatNumber> {
    fn max_size(&self) -> T;
    fn is_comparable_size(&self, other: &Self, epsilon: T) -> bool;
}

impl<T: FloatNumber> FloatRectExt<T> for FloatRect<T> {
    #[inline]
    fn max_size(&self) -> T {
        self.width().max(self.height())
    }

    #[inline]
    fn is_comparable_size(&self, other: &Self, epsilon: T) -> bool {
        let size0 = self.max_size();
        let size1 = other.max_size();
        let two = T::from_float(2.0);

        if size0 <= epsilon && size1 <= epsilon {
            return true;
        }

        size0 <= size1 * two + epsilon && size1 <= size0 * two + epsilon
    }
}
