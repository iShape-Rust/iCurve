use crate::kernel::math::rect::ToRect;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

pub(crate) trait FloatRectExt<T: FloatNumber> {
    fn max_size(&self) -> T;
}

impl<T: FloatNumber> FloatRectExt<T> for FloatRect<T> {
    #[inline]
    fn max_size(&self) -> T {
        self.width().max(self.height())
    }
}

pub(crate) trait RectIntersection<T: FloatNumber> {
    fn is_intersect_by_rects(&self, other: &Self, epsilon: T)
    where
        Self: ToRect<T>,
    {
        let r0 = self.to_rect();
        let r1 = other.to_rect();
    }
}
