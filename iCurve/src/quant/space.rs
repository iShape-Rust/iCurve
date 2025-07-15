use crate::float::math::point::Point;
use crate::int::math::point::IntPoint;

#[derive(Clone)]
pub struct Space {
    cell_size_power: u32,
    scale_to_int: f64,
    scale_to_float: f64
}

impl Space {

    #[inline]
    pub fn new(scale_power: i32, cell_size_power: u32) -> Self {
        let e = scale_power + cell_size_power as i32;
        let scale_to_int = 2f64.powi(e);
        let scale_to_float = 2f64.powi(-e);

        Self { cell_size_power, scale_to_int, scale_to_float }
    }

    #[inline(always)]
    pub fn debug() -> Self {
        Self::new(10,3)
    }

    #[inline(always)]
    fn snap_to_grid_value(&self, a: i64) -> i64 {
        let p = self.cell_size_power;
        let s = ((a << 1) >> p) & 1;
        let c = a >> p;
        (c + s) << p
    }

    #[inline(always)]
    pub fn float_val(&self, a: i64) -> f64 {
        a as f64 * self.scale_to_float
    }

    #[inline(always)]
    pub fn int_val(&self, a: f64) -> i64 {
        self.snap_to_grid_value((a * self.scale_to_int) as i64)
    }

    #[inline(always)]
    pub fn int_point(&self, point: Point) -> IntPoint {
        let x = self.int_val(point.x);
        let y = self.int_val(point.y);
        IntPoint::new(x, y)
    }

    #[inline(always)]
    pub fn float_point(&self, point: IntPoint) -> Point {
        let x = self.float_val(point.x);
        let y = self.float_val(point.y);
        Point::new(x, y)
    }
}

impl Default for Space {
    #[inline(always)]
    fn default() -> Self {
        Self::new(20,4)
    }
}

#[cfg(test)]
mod tests {
    use crate::quant::space::Space;

    #[test]
    fn test_0() {
        let space = Space::new(0, 4);
        assert_eq!(space.snap_to_grid_value(7), 0);
    }

    #[test]
    fn test_1() {
        let space = Space::new(0, 4);
        assert_eq!(space.snap_to_grid_value(8), 16);
    }

    #[test]
    fn test_2() {
        let space = Space::new(0, 4);
        assert_eq!(space.snap_to_grid_value(-7), 0);
    }

    #[test]
    fn test_3() {
        let space = Space::new(0, 4);
        assert_eq!(space.snap_to_grid_value(-9), -16);
    }
}