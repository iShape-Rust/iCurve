use i_overlay::i_float::float::number::FloatNumber;

/// Computes a finite vector's length without squaring its original magnitude.
#[inline]
pub(super) fn vector_length<F: FloatNumber>(x: F, y: F) -> F {
    let scale = x.abs().max(y.abs());
    if scale == F::ZERO {
        return F::ZERO;
    }
    // Squaring tiny axes directly can underflow even when their length is
    // representable. Normalize before squaring, then restore the magnitude.
    let x = x / scale;
    let y = y / scale;
    scale * (x * x + y * y).sqrt()
}
