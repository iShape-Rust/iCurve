use crate::int::math::point::IntPoint;

#[derive(Debug, Clone)]
pub struct IntArc {
    radius: i64, // ???
    center: IntPoint,
    start: IntPoint,
    end: IntPoint,
    direction: bool
}