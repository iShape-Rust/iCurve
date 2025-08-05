use crate::float::math::point::Point;
use eframe::egui::Pos2;

impl From<Pos2> for Point {
    #[inline]
    fn from(value: Pos2) -> Self {
        Self::new(value.x as f64, value.y as f64)
    }
}

impl From<&Pos2> for Point {
    #[inline]
    fn from(value: &Pos2) -> Self {
        Self::new(value.x as f64, value.y as f64)
    }
}

impl From<Point> for Pos2 {
    #[inline]
    fn from(value: Point) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

impl From<&Point> for Pos2 {
    #[inline]
    fn from(value: &Point) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}
