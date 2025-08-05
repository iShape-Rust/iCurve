use crate::int::math::rect::IntRect;

impl IntRect {
    #[inline(always)]
    pub(crate) fn max_log_size(&self) -> u32 {
        self.width().max(self.height()).ilog2()
    }
}

pub(crate) const FIT_32_MAX: u32 = 28;
pub(crate) const FIT_32_ATOM: u32 = 4;

pub(crate) trait Fit32 {
    fn is_fit32(&self) -> bool;
    fn is_atom(&self) -> bool;
}

impl Fit32 for u32 {
    #[inline(always)]
    fn is_fit32(&self) -> bool {
        *self < FIT_32_MAX
    }

    #[inline(always)]
    fn is_atom(&self) -> bool {
        *self < FIT_32_ATOM
    }
}