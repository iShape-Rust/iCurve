use crate::int::bezier::spline::SplitPosition;
use crate::int::math::point::IntPoint;

pub(crate) struct LineDivider {
    a: IntPoint,
    b: IntPoint,
}

impl LineDivider {
    #[inline]
    pub(crate) fn new(a: IntPoint, b: IntPoint) -> Self {
        Self { a, b }
    }

    #[inline]
    pub(crate) fn point_at(&self, position: &SplitPosition) -> IntPoint {
        let x = Self::value_at(self.a.x, self.b.x, position);
        let y = Self::value_at(self.a.y, self.b.y, position);
        IntPoint::new(x, y)
    }

    #[inline]
    fn value_at(a: i64, b: i64, position: &SplitPosition) -> i64 {
        let v = position.value as i128;
        let t = (v * (b - a) as i128) >> position.power;

        a + t as i64
    }
}

impl SplitPosition {
    #[inline]
    pub(crate) fn bisect(&self) -> (Self, Self) {
        let power = self.power + 1;
        let val_0 = self.value << 1;
        let val_1 = (self.value << 1) | 1;

        (
            Self {
                power,
                value: val_0,
            },
            Self {
                power,
                value: val_1,
            },
        )
    }
}