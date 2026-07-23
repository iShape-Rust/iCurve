use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

/// Supporting ellipse in float coordinates.
///
/// `rotation` is counter-clockwise and expressed in radians. The radii are
/// measured before this rotation is applied.
#[derive(Clone, Copy, PartialEq)]
pub struct Ellipse<P: FloatPointCompatible> {
    pub center: P,
    pub radius_x: P::Scalar,
    pub radius_y: P::Scalar,
    pub rotation: P::Scalar,
}

/// Directed interval on a supporting [`Ellipse`].
///
/// Angles are expressed in radians. The sign of `sweep_angle` defines the
/// direction: positive is counter-clockwise and negative is clockwise. A
/// single arc may span at most one full revolution.
#[derive(Clone, Copy, PartialEq)]
pub struct EllipticArc<P: FloatPointCompatible> {
    pub ellipse: Ellipse<P>,
    pub start_angle: P::Scalar,
    pub sweep_angle: P::Scalar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EllipticArcError {
    NonFinite,
    NonPositiveRadius,
    ZeroSweep,
    SweepTooLarge,
}

impl<P: FloatPointCompatible> Ellipse<P> {
    #[inline]
    pub fn point_at(&self, angle: P::Scalar) -> P {
        let (phase_sin, phase_cos) = angle.sin_cos();
        let (rotation_sin, rotation_cos) = self.rotation.sin_cos();
        let local_x = self.radius_x * phase_cos;
        let local_y = self.radius_y * phase_sin;
        let x = self.center.x() + local_x * rotation_cos - local_y * rotation_sin;
        let y = self.center.y() + local_x * rotation_sin + local_y * rotation_cos;
        P::from_xy(x, y)
    }

    #[inline]
    pub(crate) fn bounds(&self) -> FloatRect<P::Scalar> {
        let (rotation_sin, rotation_cos) = self.rotation.sin_cos();
        let axis_x_x = self.radius_x * rotation_cos;
        let axis_x_y = self.radius_x * rotation_sin;
        let axis_y_x = -self.radius_y * rotation_sin;
        let axis_y_y = self.radius_y * rotation_cos;
        let extent_x = (axis_x_x * axis_x_x + axis_y_x * axis_y_x).sqrt();
        let extent_y = (axis_x_y * axis_x_y + axis_y_y * axis_y_y).sqrt();

        FloatRect::new(
            self.center.x() - extent_x,
            self.center.x() + extent_x,
            self.center.y() - extent_y,
            self.center.y() + extent_y,
        )
    }

    #[inline]
    fn validate(&self) -> Result<(), EllipticArcError> {
        if !is_finite_point(self.center)
            || !is_finite(self.radius_x)
            || !is_finite(self.radius_y)
            || !is_finite(self.rotation)
        {
            return Err(EllipticArcError::NonFinite);
        }
        if self.radius_x <= P::Scalar::ZERO || self.radius_y <= P::Scalar::ZERO {
            return Err(EllipticArcError::NonPositiveRadius);
        }
        Ok(())
    }
}

impl<P: FloatPointCompatible> EllipticArc<P> {
    #[inline]
    pub fn start_point(&self) -> P {
        self.ellipse.point_at(self.start_angle)
    }

    #[inline]
    pub fn end_point(&self) -> P {
        if self.is_full_turn() {
            self.start_point()
        } else {
            self.ellipse.point_at(self.start_angle + self.sweep_angle)
        }
    }

    #[inline]
    pub(crate) fn validate(&self) -> Result<(), EllipticArcError> {
        self.ellipse.validate()?;
        if !is_finite(self.start_angle) || !is_finite(self.sweep_angle) {
            return Err(EllipticArcError::NonFinite);
        }
        if self.sweep_angle == P::Scalar::ZERO {
            return Err(EllipticArcError::ZeroSweep);
        }
        let tolerance = if P::Scalar::BITS <= 32 { 1.0e-6 } else { 1.0e-14 };
        if self.sweep_angle.to_f64().abs() > core::f64::consts::TAU * (1.0 + tolerance) {
            return Err(EllipticArcError::SweepTooLarge);
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn is_full_turn(&self) -> bool {
        let turns = self.sweep_angle.to_f64() / core::f64::consts::TAU;
        let tolerance = if P::Scalar::BITS <= 32 { 1.0e-6 } else { 1.0e-14 };
        (turns.abs() - 1.0).abs() <= tolerance
    }
}

#[inline]
pub(crate) fn is_finite_point<P: FloatPointCompatible>(point: P) -> bool {
    is_finite(point.x()) && is_finite(point.y())
}

#[inline]
fn is_finite<F: FloatNumber>(value: F) -> bool {
    value.to_f64().is_finite()
}
