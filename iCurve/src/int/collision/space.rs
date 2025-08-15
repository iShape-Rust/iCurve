use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct Space {
    pub line_level: u32,
    pub convex_level: u32,
}

impl Space {

    #[inline(always)]
    pub(crate) fn snap_radius(&self) -> u64 {
        1 << self.line_level
    }

    #[inline]
    pub fn with_line_level(line_level: u32) -> Self {
        let convex_level = 32 - line_level;
        Self {
            line_level,
            convex_level,
        }
    }
}

impl Default for Space {
    #[inline]
    fn default() -> Self {
        Self::with_line_level(4)
    }
}

impl IntRect {
    #[inline(always)]
    pub(crate) fn size_level(&self) -> u32 {
        self.width().max(self.height()).ilog2()
    }
}
