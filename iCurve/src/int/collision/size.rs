use crate::int::math::rect::IntRect;

pub(crate) enum ColliderSize {
    Large,
    Medium,
    Atomic
}

impl IntRect {

    #[inline]
    pub(crate) fn common_collider_size(&self, other: &Self ) -> ColliderSize {
        let m0= self.width().max(self.height());
        let m1= other.width().max(other.height());
        let max = m0.max(m1);
        ColliderSize::with_value(max.ilog2())
    }

    #[inline]
    pub(crate) fn collider_size(&self) -> ColliderSize {
        let max = self.width().max(self.height());
        ColliderSize::with_value(max.ilog2())
    }
}

impl ColliderSize {
    #[inline]
    fn with_value(value: u32) -> Self {
        if value > 28 {
            ColliderSize::Large
        } else if value > 4 {
            ColliderSize::Medium
        } else {
            ColliderSize::Atomic
        }
    }
}