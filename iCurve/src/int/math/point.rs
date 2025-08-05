use core::cmp::Ordering;
use core::{fmt, ops};
use crate::int::math::offset::IntOffset;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    pub fn dot_product(&self, other: &Self) -> i64 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross_product(&self, other: &Self) -> i64 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn sqr_dist(&self, other: &Self) -> u64 {
        let x = self.x.abs_diff(other.x);
        let y = self.y.abs_diff(other.y);
         x * x + y * y
    }

    #[inline]
    pub fn sqr_len(&self) -> u64 {
        let x = self.x.unsigned_abs();
        let y = self.y.unsigned_abs();
         x * x + y * y
    }

    #[inline]
    pub fn mid(&self, other: &Self) -> Self {
        let x = (self.x + other.x) / 2;
        let y = (self.y + other.y) / 2;
        Self { x, y }
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

impl From<[i64; 2]> for IntPoint {
    #[inline]
    fn from(value: [i64; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<IntOffset> for IntPoint {
    #[inline]
    fn from(value: IntOffset) -> Self {
        Self::new(value.x, value.y)
    }
}

impl ops::Mul<i64> for IntPoint {
    type Output = Self;

    #[inline(always)]
    fn mul(self, scalar: i64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl PartialOrd for IntPoint {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntPoint {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let x = self.x == other.x;
        if x && self.y == other.y {
            Ordering::Equal
        } else if self.x < other.x || x && self.y < other.y {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl fmt::Display for IntPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::int::math::normalize::{VectorNormalization16, VectorNormalization16Util};
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_basic_normalization() {
        let unit = VectorNormalization16Util::UNIT as i64;
        assert_eq!(IntPoint::new(unit, 0).normalized_16bit(), IntPoint::new(unit, 0));
        assert_eq!(IntPoint::new(0, unit).normalized_16bit(), IntPoint::new(0, unit));
        assert_eq!(IntPoint::new(3, 4).normalized_16bit(), IntPoint::new(39321, 52428));
    }

    #[test]
    fn test_big_numbers() {
        let x: i64 = 507758875930;
        let y: i64 = 748317763344;
        let p = IntPoint::new(x, y).normalized_16bit();
        let n = p.normalized_16bit();
        let unit = VectorNormalization16Util::UNIT as i64;
        assert!(n.x.abs() <= unit);
        assert!(n.y.abs() <= unit);

        let sqr_len = n.x * n.x + n.y * n.y;
        let error = unit * unit - sqr_len;

        assert!(error < unit * 100);
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
            let n = p.normalized_16bit();
            assert!(n.x.abs() <= VectorNormalization16Util::UNIT as i64);
            assert!(n.y.abs() <= VectorNormalization16Util::UNIT as i64);


            let sqr_len = n.x * n.x + n.y * n.y;
            let error = 1024 * 1024 - sqr_len;

            assert!(error < 1024 * 5);
        }
    }
}