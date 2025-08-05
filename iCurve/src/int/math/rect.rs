use crate::int::math::point::IntPoint;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntRect {
    pub min: IntPoint,
    pub max: IntPoint,
}

impl IntRect {

    #[inline(always)]
    pub fn width(&self) -> u64 {
        self.max.x.abs_diff(self.min.x)
    }

    #[inline(always)]
    pub fn height(&self) -> u64 {
        self.max.y.abs_diff(self.min.y)
    }

    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            min: IntPoint::new(i64::MAX, i64::MAX),
            max: IntPoint::new(i64::MIN, i64::MIN),
        }
    }

    #[inline(always)]
    pub fn with_min_max(min: IntPoint, max: IntPoint) -> Self {
        Self { min, max }
    }

    #[inline(always)]
    pub fn with_ab(a: &IntPoint, b: &IntPoint) -> Self {
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = a.x.max(b.x);
        let max_y = a.y.max(b.y);
        Self {
            min: IntPoint { x: min_x, y: min_y },
            max: IntPoint { x: max_x, y: max_y },
        }
    }

    #[inline(always)]
    pub fn with_points(points: &[IntPoint]) -> Self {
        Self::with_points_iter(points.iter())
    }

    #[inline]
    pub fn with_points_iter<'a, I: Iterator<Item = &'a IntPoint>>(iter: I) -> Self {
        let mut rect = Self::empty();
        for p in iter {
            rect.add_point(p);
        }
        rect
    }

    #[inline]
    pub fn with_rects_iter<'a, I: Iterator<Item = &'a IntRect>>(iter: I) -> Self {
        let mut rect = Self::empty();
        for r in iter {
            rect.add_rect(r);
        }
        rect
    }

    #[inline]
    pub fn add_point(&mut self, point: &IntPoint) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }

    #[inline]
    pub fn add_rect(&mut self, rect: &IntRect) {
        self.min.x = self.min.x.min(rect.min.x);
        self.min.y = self.min.y.min(rect.min.y);
        self.max.x = self.max.x.max(rect.max.x);
        self.max.y = self.max.y.max(rect.max.y);
    }

    #[inline]
    pub fn is_intersect_border_include(&self, other: &Self) -> bool {
        let x = self.min.x <= other.max.x && self.max.x >= other.min.x;
        let y = self.min.y <= other.max.y && self.max.y >= other.min.y;
        x && y
    }

    #[inline]
    pub fn is_intersect_border_exclude(&self, other: &Self) -> bool {
        let x = self.min.x < other.max.x && self.max.x > other.min.x;
        let y = self.min.y < other.max.y && self.max.y > other.min.y;
        x && y
    }

    #[inline]
    pub fn is_not_overlap(&self, other: &Self) -> bool {
        let x = self.min.x > other.max.x || other.min.x > self.max.x;
        let y = self.min.y > other.max.y || other.min.y > self.max.y;
        x || y
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use crate::int::math::point::IntPoint;
    use crate::int::math::rect::IntRect;

    #[test]
    fn test_0() {
        let rect = IntRect::with_points(&vec![
            IntPoint::new(0, 0),
            IntPoint::new(-7, 10),
            IntPoint::new(20, -5),
        ]);

        assert_eq!(rect.min.x, -7);
        assert_eq!(rect.max.x, 20);
        assert_eq!(rect.min.y, -5);
        assert_eq!(rect.max.y, 10);
    }

    #[test]
    fn test_1() {
        let a = IntRect::with_points(&vec![IntPoint::new(0, 0), IntPoint::new(10, 10)]);

        let b = IntRect::with_points(&vec![IntPoint::new(10, 10), IntPoint::new(20, 0)]);

        assert_eq!(a.is_intersect_border_exclude(&b), false);
        assert_eq!(a.is_intersect_border_include(&b), true);
    }
}