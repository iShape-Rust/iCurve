use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

pub struct EllipticArc<P: FloatPointCompatible> {
    pub center: P,
    pub radii: P,
    pub rotation: P::Scalar,
    pub start_angle: P::Scalar,
    pub sweep_angle: P::Scalar,
}

impl<P: FloatPointCompatible> EllipticArc<P> {
    pub(crate) fn end_point(&self) -> P {
        let angle = self.start_angle + self.sweep_angle;
        let x = self.radii.x() * angle.cos();
        let y = self.radii.y() * angle.sin();
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();

        P::from_xy(
            self.center.x() + x * cos - y * sin,
            self.center.y() + x * sin + y * cos,
        )
    }
}
