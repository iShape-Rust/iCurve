use crate::int::math::point::IntPoint;

pub(crate) struct LineDivider {
    a: IntPoint,
    b: IntPoint
}

impl LineDivider {

    #[inline]
    pub(crate) fn new(a: IntPoint, b: IntPoint) -> Self {
        Self { a, b }
    }

    #[inline]
    pub(crate) fn split_at(&self, step: usize, split_factor: u32) -> IntPoint {
        let x = Self::split_one_dimension_at(self.a.x, self.b.x, step, split_factor);
        let y = Self::split_one_dimension_at(self.a.y, self.b.y, step, split_factor);
        IntPoint::new(x, y)
    }

    #[inline]
    fn split_one_dimension_at(a: i64, b: i64, step: usize, split_factor: u32) -> i64 {
        let ab = b.wrapping_sub(a) as i128;
        let step = step as i128;

        let scaled = (ab * step) >> split_factor;

        a.wrapping_add(scaled as i64)
    }
}