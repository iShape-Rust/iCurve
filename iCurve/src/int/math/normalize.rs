use crate::int::math::point::IntPoint;

pub trait Normalize16 {
    fn normalized_16bit(&self) -> IntPoint;
}

pub const VALUABLE_BITS: u32 = 16;

pub const UNIT: u32 = 1 << VALUABLE_BITS;

pub fn normalize_unit_value(fract: f64) -> u32 {
    (fract * UNIT as f64) as u32
}

impl Normalize16 for IntPoint {
    fn normalized_16bit(&self) -> IntPoint {
        let dx = (self.x as i128).unsigned_abs().pow(2);
        let dy = (self.y as i128).unsigned_abs().pow(2);
        let sqr_len = dx + dy;
        if sqr_len == 0 {
            return IntPoint::zero()
        }

        let bits_count = sqr_len.ilog2();

        let len = sqr_len.isqrt() as i64;

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
}