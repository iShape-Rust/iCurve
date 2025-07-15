use crate::int::math::offset::IntOffset;
use crate::int::math::point::IntPoint;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct IntBezierAnchor {
    pub point: IntPoint,
    pub handle_in: Option<IntOffset>,
    pub handle_out: Option<IntOffset>,
}

impl IntBezierAnchor {
    #[inline]
    pub fn handle_in_point(&self) -> Option<IntPoint> {
        self.handle_in.map(|offset| self.point + offset)
    }

    #[inline]
    pub fn handle_out_point(&self) -> Option<IntPoint> {
        self.handle_out.map(|offset| self.point + offset)
    }
}
