#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LineRange {
    pub(crate) min: i64,
    pub(crate) max: i64,
}

impl LineRange {
    #[inline(always)]
    pub(crate) fn new(a: i64, b: i64) -> Self {
        let (min, max) = if a <= b { (a, b) } else { (b, a) };
        Self { min, max }
    }

    #[inline(always)]
    pub(crate) fn is_overlap(&self, other: &Self) -> bool {
        self.min <= other.max && self.max >= other.min
    }

    #[inline(always)]
    pub(crate) fn is_not_overlap(&self, other: &Self) -> bool {
        self.min > other.max || self.max < other.min
    }
}
