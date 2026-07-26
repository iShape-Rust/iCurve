pub mod bool;
pub mod curve;

/// Bits reserved for intermediate polynomial coefficient growth.
pub(crate) const CURVE_COORDINATE_SAFETY_BITS: u32 = 6;
