use crate::int::math::point::IntPoint;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntOffset {
    pub x: i64,
    pub y: i64,
}

impl IntOffset {
    #[inline]
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
    #[inline]
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl From<IntPoint> for IntOffset {
    #[inline]
    fn from(value: IntPoint) -> Self {
        Self::new(value.x, value.y)
    }
}