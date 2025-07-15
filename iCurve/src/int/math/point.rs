use std::ops;
use std::ops::Mul;
use crate::int::math::offset::IntOffset;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntPoint {
    pub x: i64,
    pub y: i64,
}

impl IntPoint {
    #[inline]
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    #[inline]
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn normalized_10bit(&self) -> IntPoint {
        // return unit vector but scaled by 1024 (2^10)

        let dx = (self.x as i128).unsigned_abs().pow(2);
        let dy = (self.y as i128).unsigned_abs().pow(2);
        let sqr_len = dx + dy;
        if sqr_len == 0 {
            return IntPoint::new(1024, 0);
        }

        let bits_count = sqr_len.ilog2();

        let len = sqr_len.isqrt() as i64;

        const VALUABLE_BITS: u32 = 10;
        const MAX_SAFE_BITS: u32 = 63 - VALUABLE_BITS;

        if bits_count <= MAX_SAFE_BITS {
            let x = (self.x << VALUABLE_BITS) / len;
            let y = (self.y << VALUABLE_BITS) / len;
            IntPoint::new(x, y)
        } else {
            let len = len >> VALUABLE_BITS;
            let x = self.x / len;
            let y = self.y / len;
            IntPoint::new(x, y)
        }
    }

    #[inline]
    pub fn dot_product(&self, other: &Self) -> i64 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross_product(&self, other: &Self) -> i64 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn accurate_cross_product(&self, other: &Self) -> i128 {
        let x0 = self.x as i128;
        let y0 = self.y as i128;
        let x1 = other.x as i128;
        let y1 = other.y as i128;

        x0 * y1 - y0 * x1
    }

    #[inline]
    pub fn accurate_dot_product(&self, other: &Self) -> i128 {
        let x0 = self.x as i128;
        let y0 = self.y as i128;
        let x1 = other.x as i128;
        let y1 = other.y as i128;

        x0 * x1 + y0 * y1
    }
}

impl ops::Add for IntPoint {
    type Output = IntPoint;

    #[inline(always)]
    fn add(self, other: IntPoint) -> IntPoint {
        IntPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Add<IntOffset> for IntPoint {
    type Output = IntPoint;

    #[inline(always)]
    fn add(self, other: IntOffset) -> IntPoint {
        IntPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for IntPoint {
    type Output = IntPoint;

    #[inline(always)]
    fn sub(self, other: IntPoint) -> IntPoint {
        IntPoint {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl ops::Sub<IntOffset> for IntPoint {
    type Output = IntPoint;

    #[inline(always)]
    fn sub(self, other: IntOffset) -> IntPoint {
        IntPoint {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl From<IntOffset> for IntPoint {
    #[inline]
    fn from(value: IntOffset) -> Self {
        Self::new(value.x, value.y)
    }
}

impl Mul<i64> for IntPoint {
    type Output = Self;

    #[inline(always)]
    fn mul(self, scalar: i64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IntPoint;
    use rand::Rng;

    #[test]
    fn test_basic_normalization() {
        assert_eq!(IntPoint::new(1024, 0).normalized_10bit(), IntPoint::new(1024, 0));
        assert_eq!(IntPoint::new(0, 1024).normalized_10bit(), IntPoint::new(0, 1024));
        assert_eq!(IntPoint::new(3, 4).normalized_10bit(), IntPoint::new(614, 819));
    }

    #[test]
    fn test_big_numbers() {
        let x: i64 = 507758875930;
        let y: i64 = 748317763344;
        let p = IntPoint::new(x, y).normalized_10bit();
        let n = p.normalized_10bit();
        assert!(n.x.abs() <= 1024);
        assert!(n.y.abs() <= 1024);

        let sqr_len = n.x * n.x + n.y * n.y;
        let error = 1024 * 1024 - sqr_len;

        assert!(error < 1024 * 100);
    }

    #[test]
    fn test_random_normalization_accuracy() {
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let x = rng.random_range(-1_000_000_000_000..=1_000_000_000_000);
            let y = rng.random_range(-1_000_000_000_000..=1_000_000_000_000);
            if x == 0 && y == 0 {
                continue;
            }
            let p = IntPoint::new(x, y);
            let n = p.normalized_10bit();
            assert!(n.x.abs() <= 1024);
            assert!(n.y.abs() <= 1024);


            let sqr_len = n.x * n.x + n.y * n.y;
            let error = 1024 * 1024 - sqr_len;

            assert!(error < 1024 * 5);
        }
    }
}