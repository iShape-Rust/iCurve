use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::rect::FloatRect;

/// Supporting ellipse in float coordinates.
///
/// `rotation` is counter-clockwise and expressed in radians. The radii are
/// measured before this rotation is applied.
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipticArc<P: FloatPointCompatible> {
    pub ellipse: Ellipse<P>,
    pub start_angle: P::Scalar,
    pub sweep_angle: P::Scalar,
}

/// Rational quadratic representation of an elliptic arc.
///
/// `control_points` and `weights` are the authoritative geometry. The
/// supporting elliptic arc is retained as semantic metadata, but after a
/// boolean operation or coordinate snap its endpoints are not required to lie
/// exactly on that ellipse.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RationalArc<P: FloatPointCompatible> {
    pub ellipse: Ellipse<P>,
    pub control_points: [P; 3],
    pub weights: [P::Scalar; 3],
    pub start_angle: P::Scalar,
    pub sweep_angle: P::Scalar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EllipticArcError {
    NonFinite,
    NonPositiveRadius,
    ZeroSweep,
    SweepTooLarge,
    DegeneratePiece,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RationalArcError {
    Elliptic(EllipticArcError),
    NonFiniteControlPoint,
    NonFiniteWeight,
    NonPositiveWeight,
}

impl From<EllipticArcError> for RationalArcError {
    fn from(error: EllipticArcError) -> Self {
        Self::Elliptic(error)
    }
}

impl core::fmt::Display for EllipticArcError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::NonFinite => "ellipse and arc values must be finite",
            Self::NonPositiveRadius => "ellipse radii must be positive",
            Self::ZeroSweep => "arc sweep must be non-zero",
            Self::SweepTooLarge => "arc sweep must not exceed one full turn",
            Self::DegeneratePiece => "arc cannot be represented by a finite rational quadratic",
        };
        formatter.write_str(message)
    }
}

impl core::error::Error for EllipticArcError {}

impl core::fmt::Display for RationalArcError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Elliptic(error) => error.fmt(formatter),
            Self::NonFiniteControlPoint => formatter.write_str("arc control points must be finite"),
            Self::NonFiniteWeight => formatter.write_str("arc weights must be finite"),
            Self::NonPositiveWeight => formatter.write_str("arc weights must be positive"),
        }
    }
}

impl core::error::Error for RationalArcError {}

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

    /// Converts this semantic elliptic arc into XY-monotone rational
    /// quadratic pieces.
    pub fn to_rational_arcs(&self) -> Result<Vec<RationalArc<P>>, EllipticArcError> {
        self.validate()?;

        let frame = FloatEllipseFrame::new(self.ellipse);
        let cuts = collect_arc_cuts(self, &frame);
        let direction = self.sweep_angle.signum();
        let mut result = Vec::with_capacity(cuts.len().saturating_sub(1));
        let mut start_point = self.start_point();

        for index in 0..cuts.len() - 1 {
            let start_cut = cuts[index];
            let end_cut = cuts[index + 1];
            let dot = start_cut
                .phase
                .dot(end_cut.phase)
                .max(-P::Scalar::ONE)
                .min(P::Scalar::ONE);
            let denominator = P::Scalar::ONE + dot;
            if denominator <= P::Scalar::ZERO {
                return Err(EllipticArcError::DegeneratePiece);
            }

            let control_phase = FloatArcPhase {
                cos: (start_cut.phase.cos + end_cut.phase.cos) / denominator,
                sin: (start_cut.phase.sin + end_cut.phase.sin) / denominator,
            };
            let end_point = if index + 2 == cuts.len() {
                self.end_point()
            } else {
                frame.point_at(end_cut.phase)
            };
            let start_angle = self.start_angle + direction * P::Scalar::from_float(start_cut.progress);
            let sweep_angle = direction * P::Scalar::from_float(end_cut.progress - start_cut.progress);

            result.push(RationalArc {
                ellipse: self.ellipse,
                control_points: [start_point, frame.point_at(control_phase), end_point],
                weights: [
                    P::Scalar::ONE,
                    (denominator / P::Scalar::TWO).sqrt(),
                    P::Scalar::ONE,
                ],
                start_angle,
                sweep_angle,
            });
            start_point = end_point;
        }

        Ok(result)
    }
}

impl<P: FloatPointCompatible> RationalArc<P> {
    #[inline]
    pub fn start_point(&self) -> P {
        self.control_points[0]
    }

    #[inline]
    pub fn end_point(&self) -> P {
        self.control_points[2]
    }

    /// Evaluates the authoritative rational quadratic geometry.
    pub fn point_at(&self, t: P::Scalar) -> P {
        let one_minus_t = P::Scalar::ONE - t;
        let factors = [
            self.weights[0] * one_minus_t * one_minus_t,
            self.weights[1] * P::Scalar::TWO * one_minus_t * t,
            self.weights[2] * t * t,
        ];
        let denominator = factors[0] + factors[1] + factors[2];
        let x = (self.control_points[0].x() * factors[0]
            + self.control_points[1].x() * factors[1]
            + self.control_points[2].x() * factors[2])
            / denominator;
        let y = (self.control_points[0].y() * factors[0]
            + self.control_points[1].y() * factors[1]
            + self.control_points[2].y() * factors[2])
            / denominator;
        P::from_xy(x, y)
    }

    /// Returns the supporting ellipse interval.
    ///
    /// This preserves semantic arc metadata, but it is not necessarily
    /// geometrically identical to the rational curve after snapping.
    #[inline]
    pub fn supporting_arc(&self) -> EllipticArc<P> {
        EllipticArc {
            ellipse: self.ellipse,
            start_angle: self.start_angle,
            sweep_angle: self.sweep_angle,
        }
    }

    /// Returns an elliptic arc only when the rational geometry still matches
    /// its supporting ellipse within a dimensionless tolerance.
    ///
    /// Snapping performed by boolean operations can make this return `None`.
    pub fn try_to_elliptic_arc(&self, tolerance: P::Scalar) -> Option<EllipticArc<P>> {
        if self.validate().is_err() || !is_finite(tolerance) || tolerance < P::Scalar::ZERO {
            return None;
        }

        let arc = self.supporting_arc();
        let samples = [
            P::Scalar::ZERO,
            P::Scalar::from_float(0.25_f64),
            P::Scalar::HALF,
            P::Scalar::from_float(0.75_f64),
            P::Scalar::ONE,
        ];
        if samples
            .iter()
            .any(|t| !arc.ellipse.contains_with_tolerance(self.point_at(*t), tolerance))
        {
            return None;
        }

        let endpoint_tolerance = tolerance * tolerance;
        if arc
            .ellipse
            .normalized_distance_squared(self.start_point(), arc.start_point())
            > endpoint_tolerance
            || arc
                .ellipse
                .normalized_distance_squared(self.end_point(), arc.end_point())
                > endpoint_tolerance
        {
            return None;
        }

        Some(arc)
    }

    pub(crate) fn validate(&self) -> Result<(), RationalArcError> {
        self.supporting_arc().validate()?;
        if self.control_points.iter().any(|point| !is_finite_point(*point)) {
            return Err(RationalArcError::NonFiniteControlPoint);
        }
        if self.weights.iter().any(|weight| !is_finite(*weight)) {
            return Err(RationalArcError::NonFiniteWeight);
        }
        if self.weights.iter().any(|weight| *weight <= P::Scalar::ZERO) {
            return Err(RationalArcError::NonPositiveWeight);
        }
        Ok(())
    }
}

impl<P: FloatPointCompatible> Ellipse<P> {
    fn normalized_distance_squared(&self, a: P, b: P) -> P::Scalar {
        let dx = a.x() - b.x();
        let dy = a.y() - b.y();
        let (sin, cos) = self.rotation.sin_cos();
        let local_x = (dx * cos + dy * sin) / self.radius_x;
        let local_y = (-dx * sin + dy * cos) / self.radius_y;
        local_x * local_x + local_y * local_y
    }

    fn contains_with_tolerance(&self, point: P, tolerance: P::Scalar) -> bool {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let (sin, cos) = self.rotation.sin_cos();
        let local_x = (dx * cos + dy * sin) / self.radius_x;
        let local_y = (-dx * sin + dy * cos) / self.radius_y;
        let residual = (local_x * local_x + local_y * local_y - P::Scalar::ONE).abs();
        residual <= tolerance
    }
}

#[derive(Clone, Copy)]
struct ArcCut<F: FloatNumber> {
    progress: f64,
    phase: FloatArcPhase<F>,
}

fn collect_arc_cuts<P: FloatPointCompatible>(
    arc: &EllipticArc<P>,
    frame: &FloatEllipseFrame<P>,
) -> Vec<ArcCut<P::Scalar>> {
    let start = FloatArcPhase::from_angle(arc.start_angle);
    let end = if arc.is_full_turn() {
        start
    } else {
        FloatArcPhase::from_angle(arc.start_angle + arc.sweep_angle)
    };
    let counter_clockwise = arc.sweep_angle > P::Scalar::ZERO;
    let sweep = arc.sweep_angle.to_f64().abs();
    let tolerance = if P::Scalar::BITS <= 32 { 1.0e-5 } else { 1.0e-12 };
    let mut cuts = Vec::with_capacity(6);
    cuts.push(ArcCut {
        progress: 0.0,
        phase: start,
    });

    for phase in frame.extremum_phases() {
        let progress = directed_progress(start, phase, counter_clockwise);
        if progress > tolerance && progress < sweep - tolerance {
            cuts.push(ArcCut { progress, phase });
        }
    }

    cuts[1..].sort_by(|a, b| {
        a.progress
            .partial_cmp(&b.progress)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    cuts.dedup_by(|a, b| (a.progress - b.progress).abs() <= tolerance);
    cuts.push(ArcCut {
        progress: sweep,
        phase: end,
    });
    cuts
}

fn directed_progress<F: FloatNumber>(
    start: FloatArcPhase<F>,
    end: FloatArcPhase<F>,
    counter_clockwise: bool,
) -> f64 {
    let dot = start.dot(end).max(-F::ONE).min(F::ONE);
    let angle = dot.acos().to_f64();
    let oriented_cross = if counter_clockwise {
        start.cross(end)
    } else {
        -start.cross(end)
    };

    if oriented_cross < F::ZERO {
        core::f64::consts::TAU - angle
    } else {
        angle
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FloatArcPhase<F: FloatNumber> {
    pub(crate) cos: F,
    pub(crate) sin: F,
}

impl<F: FloatNumber> FloatArcPhase<F> {
    pub(crate) fn from_angle(angle: F) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::normalized(cos, sin)
    }

    fn normalized(cos: F, sin: F) -> Self {
        let length = (cos * cos + sin * sin).sqrt();
        Self {
            cos: cos / length,
            sin: sin / length,
        }
    }

    fn opposite(self) -> Self {
        Self {
            cos: -self.cos,
            sin: -self.sin,
        }
    }

    pub(crate) fn dot(self, other: Self) -> F {
        self.cos * other.cos + self.sin * other.sin
    }

    fn cross(self, other: Self) -> F {
        self.cos * other.sin - self.sin * other.cos
    }
}

pub(crate) struct FloatEllipseFrame<P: FloatPointCompatible> {
    pub(crate) center: P,
    pub(crate) axis_x_x: P::Scalar,
    pub(crate) axis_x_y: P::Scalar,
    pub(crate) axis_y_x: P::Scalar,
    pub(crate) axis_y_y: P::Scalar,
}

impl<P: FloatPointCompatible> FloatEllipseFrame<P> {
    pub(crate) fn new(ellipse: Ellipse<P>) -> Self {
        let (rotation_sin, rotation_cos) = ellipse.rotation.sin_cos();
        Self {
            center: ellipse.center,
            axis_x_x: ellipse.radius_x * rotation_cos,
            axis_x_y: ellipse.radius_x * rotation_sin,
            axis_y_x: -ellipse.radius_y * rotation_sin,
            axis_y_y: ellipse.radius_y * rotation_cos,
        }
    }

    fn point_at(&self, phase: FloatArcPhase<P::Scalar>) -> P {
        let x = self.center.x() + self.axis_x_x * phase.cos + self.axis_y_x * phase.sin;
        let y = self.center.y() + self.axis_x_y * phase.cos + self.axis_y_y * phase.sin;
        P::from_xy(x, y)
    }

    fn extremum_phases(&self) -> [FloatArcPhase<P::Scalar>; 4] {
        let x = FloatArcPhase::normalized(self.axis_x_x, self.axis_y_x);
        let y = FloatArcPhase::normalized(self.axis_x_y, self.axis_y_y);
        [x, x.opposite(), y, y.opposite()]
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

#[cfg(test)]
mod tests {
    use super::*;

    type Point = [f64; 2];

    fn full_ellipse() -> EllipticArc<Point> {
        EllipticArc {
            ellipse: Ellipse {
                center: [2.0, -3.0],
                radius_x: 8.0,
                radius_y: 3.0,
                rotation: 0.4,
            },
            start_angle: 0.2,
            sweep_angle: core::f64::consts::TAU,
        }
    }

    #[test]
    fn elliptic_arc_materializes_as_connected_rational_arcs() {
        let source = full_ellipse();
        let arcs = source.to_rational_arcs().expect("valid ellipse");

        assert!(arcs.len() >= 4);
        assert_eq!(arcs[0].start_point(), source.start_point());
        assert_eq!(arcs.last().unwrap().end_point(), source.end_point());
        for pair in arcs.windows(2) {
            assert_eq!(pair[0].end_point(), pair[1].start_point());
        }
        for arc in arcs {
            assert_eq!(arc.point_at(0.0), arc.start_point());
            assert_eq!(arc.point_at(1.0), arc.end_point());
            assert!(arc.try_to_elliptic_arc(1.0e-10).is_some());
        }
    }

    #[test]
    fn snapped_rational_arc_is_not_reported_as_exact_elliptic_arc() {
        let mut arc = full_ellipse().to_rational_arcs().expect("valid ellipse")[0];
        arc.control_points[2][0] += 0.1;

        assert!(arc.try_to_elliptic_arc(1.0e-10).is_none());
        assert!(arc.try_to_elliptic_arc(0.1).is_some());
    }
}
