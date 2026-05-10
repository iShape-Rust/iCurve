use i_overlay::i_float::float::compatible::FloatPointCompatible;

pub struct EllipticArc<P: FloatPointCompatible> {
    pub center: P,
    pub radii: P,
    pub rotation: P::Scalar,
    pub start_angle: P::Scalar,
    pub sweep_angle: P::Scalar,
}
