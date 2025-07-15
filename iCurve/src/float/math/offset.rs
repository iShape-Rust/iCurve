use crate::float::math::point::Point;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Offset {
    pub x: f64,
    pub y: f64,
}

impl Offset {
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<Point> for Offset {
    #[inline]
    fn from(value: Point) -> Self {
        Self::new(value.x, value.y)
    }
}