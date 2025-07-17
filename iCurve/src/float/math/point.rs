use core::ops;
use libm::sqrt;
use crate::float::math::offset::Offset;
use crate::int::math::point::IntPoint;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn sqr_length(&self) -> f64 {
        let dx = self.x;
        let dy = self.y;

        dx * dx + dy * dy
    }

    #[inline]
    pub fn length(&self) -> f64 {
        sqrt(self.sqr_length())
    }

    #[inline]
    pub fn distance(&self, other: Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        sqrt(dx * dx + dy * dy)
    }

    #[inline]
    pub fn normalized(&self) -> Point {
        let inv_len = 1.0 / self.length();
        let x = self.x * inv_len;
        let y = self.y * inv_len;
        Point { x, y }
    }

    #[inline]
    pub fn dot_product(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl From<Offset> for Point {
    #[inline]
    fn from(value: Offset) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<IntPoint> for Point {
    #[inline]
    fn from(value: IntPoint) -> Self {
        Self::new(value.x as f64, value.y as f64)
    }
}

impl ops::Add for Point {
    type Output = Point;

    #[inline(always)]
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Add<Offset> for Point {
    type Output = Point;

    #[inline(always)]
    fn add(self, other: Offset) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for Point {
    type Output = Point;

    #[inline(always)]
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::Sub<Offset> for Point {
    type Output = Point;

    #[inline(always)]
    fn sub(self, other: Offset) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::AddAssign<Self> for Point {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::SubAssign<Self> for Point {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::Mul<f64> for Point {
    type Output = Self;

    #[inline(always)]
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}
