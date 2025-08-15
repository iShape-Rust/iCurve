use core::cmp::Ordering;
use crate::int::math::point::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XSegment {
    pub a: IntPoint,
    pub b: IntPoint,
}

impl PartialOrd for XSegment {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for XSegment {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.a.cmp(&other.a);
        if a == Ordering::Equal {
            self.b.cmp(&other.b)
        } else {
            a
        }
    }
}

impl XSegment {
    #[inline(always)]
    pub fn new(a: IntPoint, b: IntPoint) -> Self {
        if a < b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

impl From<[IntPoint; 2]> for XSegment {
    #[inline(always)]
    fn from(value: [IntPoint; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<&[IntPoint; 2]> for XSegment {
    #[inline(always)]
    fn from(value: &[IntPoint; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl Default for XSegment {
    fn default() -> Self {
        XSegment { a: IntPoint::zero(), b: IntPoint::zero() }
    }
}
