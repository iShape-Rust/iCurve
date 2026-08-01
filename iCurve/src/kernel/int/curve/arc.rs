use i_overlay::i_float::int::number::fixed_scale::FixedScale;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::uint::UIntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

use crate::kernel::int::curve::param::SegmentParam;

/// Compact vector used to store an ellipse semi-axis.
///
/// Arithmetic should widen components to `I::Wide` before multiplying them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ArcVector<I: IntNumber> {
    /// World-space X component of the semi-axis.
    pub x: I,
    /// World-space Y component of the semi-axis.
    pub y: I,
}

/// Fixed-point direction on the normalized unit circle.
///
/// This is not the world-space vector from the ellipse center to an endpoint.
/// A world-space point is obtained from an [`EllipseFrame`] as:
///
/// `center + axis_x * cos + axis_y * sin`.
///
/// `cos` and `sin` use [`FixedScale::<I>::DENOMINATOR`] as one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ArcPhase<I: IntNumber> {
    /// Fixed-point cosine of the ellipse parameter.
    pub cos: I,
    /// Fixed-point sine of the ellipse parameter.
    pub sin: I,
}

/// Directed traversal of an arc between two phases.
///
/// A normalized kernel arc always represents the directed minor interval
/// between its phases. Full ellipses and arcs spanning 180 degrees or more
/// must be decomposed before constructing [`ArcSegment`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ArcDirection {
    /// Traverses decreasing ellipse phase.
    Clockwise,
    /// Traverses increasing ellipse phase.
    #[default]
    CounterClockwise,
}

impl ArcDirection {
    /// Returns the opposite traversal direction.
    #[inline]
    pub fn reversed(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

/// Canonical supporting ellipse for one or more arc segments.
///
/// The intended invariant is that `axis_x` and `axis_y` are the perpendicular
/// semi-axes of the ellipse. A circle is canonicalized with world-aligned axes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct EllipseFrame<I: IntNumber> {
    /// Center of the supporting ellipse.
    pub center: IntPoint<I>,
    /// First semi-axis. It corresponds to phase `(1, 0)`.
    pub axis_x: ArcVector<I>,
    /// Second semi-axis. It corresponds to phase `(0, 1)`.
    pub axis_y: ArcVector<I>,
}

/// Integer kernel representation reserved for an elliptic arc.
///
/// Initial arcs will be split at every world-space X/Y extremum. The rational
/// quadratic data is stored explicitly so subsequent splits can use rational
/// de Casteljau without trigonometry. All weights are positive fixed-point
/// values; after a split they are not required to have normalized endpoints.
///
/// `ellipse`, phases, and `direction` preserve the semantic identity of the
/// source arc. `control_points` and `weights` are the authoritative geometry
/// used by kernel algorithms. Boolean overlay processing may snap a shared endpoint,
/// so the rational endpoints are not required to lie exactly on `ellipse`
/// after every operation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ArcSegment<I: IntNumber> {
    /// Supporting ellipse shared by all subsegments of the source arc.
    pub ellipse: EllipseFrame<I>,
    /// Rational quadratic points: start, tangent control, and end.
    pub control_points: [IntPoint<I>; 3],
    /// Positive fixed-point rational weights corresponding to control points.
    ///
    /// An initial normalized arc normally has `[ONE, middle, ONE]`. Rational
    /// de Casteljau produces three general weights, so endpoint weights must
    /// not be assumed to remain equal to one.
    pub weights: [I; 3],
    /// Normalized-circle phase associated with `control_points[0]`.
    pub start_phase: ArcPhase<I>,
    /// Normalized-circle phase associated with `control_points[2]`.
    pub end_phase: ArcPhase<I>,
    /// Traversal direction from `start_phase` to `end_phase`.
    pub direction: ArcDirection,
}

impl<I: IntNumber> ArcPhase<I> {
    #[inline]
    fn from_wide_vector(cos: I::Wide, sin: I::Wide) -> Option<Self> {
        let max_abs = cos.unsigned_abs().max(sin.unsigned_abs());
        if max_abs == I::WideUInt::ZERO {
            return None;
        }

        // Squaring raw components may exceed `I::Wide`. Move both by the
        // same power of two so the sum of squares is safe and uses as much
        // precision as the widened representation permits.
        let bit_length = I::WideUInt::BITS - max_abs.leading_zeros();
        let target_bits = I::WideUInt::HALF_BITS - 1;
        let (scaled_cos, scaled_sin) = if bit_length > target_bits {
            let shift = bit_length - target_bits;
            (cos.shr_round(shift), sin.shr_round(shift))
        } else {
            let shift = target_bits - bit_length;
            (cos << shift, sin << shift)
        };
        let length = (scaled_cos * scaled_cos + scaled_sin * scaled_sin).isqrt();
        if length == I::Wide::ZERO {
            return None;
        }

        Some(Self {
            cos: I::from_wide(FixedScale::<I>::div_to_scaled_round(scaled_cos, length)),
            sin: I::from_wide(FixedScale::<I>::div_to_scaled_round(scaled_sin, length)),
        })
    }

    #[inline]
    fn interpolated(self, other: Self, t: SegmentParam<I>) -> Option<Self> {
        let denominator = SegmentParam::<I>::DENOMINATOR;
        let t = t.value();
        let one_minus_t = denominator - t;
        let cos = one_minus_t * self.cos.to_wide() + t * other.cos.to_wide();
        let sin = one_minus_t * self.sin.to_wide() + t * other.sin.to_wide();

        Self::from_wide_vector(cos, sin)
    }

    #[inline]
    fn debug_assert_invariants(&self) {
        let denominator = FixedScale::<I>::DENOMINATOR;
        let cos = self.cos.to_wide();
        let sin = self.sin.to_wide();
        let square_length = cos * cos + sin * sin;
        let expected = denominator * denominator;
        let error = (square_length - expected).unsigned_abs();
        let tolerance = (denominator << 2).unsigned_abs();

        debug_assert!(error <= tolerance, "arc phase must be a fixed-point unit vector");
    }
}

impl<I: IntNumber> EllipseFrame<I> {
    #[inline]
    fn debug_assert_invariants(&self) {
        let axis_x_x = self.axis_x.x.to_wide();
        let axis_x_y = self.axis_x.y.to_wide();
        let axis_y_x = self.axis_y.x.to_wide();
        let axis_y_y = self.axis_y.y.to_wide();

        debug_assert!(
            axis_x_x != I::Wide::ZERO || axis_x_y != I::Wide::ZERO,
            "ellipse first semi-axis must be non-zero"
        );
        debug_assert!(
            axis_y_x != I::Wide::ZERO || axis_y_y != I::Wide::ZERO,
            "ellipse second semi-axis must be non-zero"
        );
        debug_assert!(
            axis_x_x * axis_y_y - axis_x_y * axis_y_x != I::Wide::ZERO,
            "ellipse frame must be non-degenerate"
        );

        // Exact perpendicularity is intentionally not asserted: converting
        // a rotated float ellipse to integer axes can introduce a small dot
        // product through coordinate quantization.
    }

    /// Recovers a normalized ellipse phase for a world-space point.
    ///
    /// The inverse frame transform gives a vector proportional to `(cos, sin)`.
    /// Its common determinant is irrelevant after normalization, which keeps
    /// the operation entirely in integer arithmetic.
    #[inline]
    pub(crate) fn phase_at(&self, point: IntPoint<I>) -> Option<ArcPhase<I>> {
        let vx = point.x.to_wide() - self.center.x.to_wide();
        let vy = point.y.to_wide() - self.center.y.to_wide();
        let ax_x = self.axis_x.x.to_wide();
        let ax_y = self.axis_x.y.to_wide();
        let ay_x = self.axis_y.x.to_wide();
        let ay_y = self.axis_y.y.to_wide();

        let determinant = ax_x * ay_y - ax_y * ay_x;
        debug_assert!(determinant != I::Wide::ZERO);
        if determinant == I::Wide::ZERO {
            return None;
        }

        let mut cos = vx * ay_y - vy * ay_x;
        let mut sin = ax_x * vy - ax_y * vx;
        if determinant < I::Wide::ZERO {
            cos = -cos;
            sin = -sin;
        }

        ArcPhase::from_wide_vector(cos, sin)
    }
}

impl<I: IntNumber> ArcSegment<I> {
    /// Returns whether the rational control polygon is weakly monotone on
    /// both world axes.
    ///
    /// Positive rational weights make this a sufficient condition for the
    /// represented curve to be XY-monotone.
    #[inline]
    pub(crate) fn is_xy_monotone(&self) -> bool {
        let [p0, p1, p2] = self.control_points;
        is_ordered(p0.x, p1.x, p2.x) && is_ordered(p0.y, p1.y, p2.y)
    }

    /// Evaluates the rational quadratic with fixed-point de Casteljau.
    #[inline]
    pub fn point_at(&self, t: SegmentParam<I>) -> IntPoint<I> {
        self.rational_levels(t).2.point
    }

    /// Reverses the arc while preserving its supporting ellipse.
    #[inline]
    pub fn reverse(&mut self) {
        self.control_points.reverse();
        self.weights.reverse();
        core::mem::swap(&mut self.start_phase, &mut self.end_phase);
        self.direction = self.direction.reversed();
    }

    /// Returns a reversed copy of this arc.
    #[inline]
    pub fn reversed(mut self) -> Self {
        self.reverse();
        self
    }

    #[inline]
    pub(crate) fn rational_split(&self, t: SegmentParam<I>) -> [Self; 2] {
        let (q01, q12, q012) = self.rational_levels(t);
        let split_phase = if t.value() == I::Wide::ZERO {
            self.start_phase
        } else if t.value() == SegmentParam::<I>::DENOMINATOR {
            self.end_phase
        } else {
            // A very coarse coordinate grid can quantize the evaluated point
            // to the ellipse center. Preserve a usable semantic phase in that
            // case by interpolating the directed minor interval.
            self.ellipse
                .phase_at(q012.point)
                .or_else(|| self.start_phase.interpolated(self.end_phase, t))
                .unwrap_or(self.start_phase)
        };

        [
            Self {
                ellipse: self.ellipse,
                control_points: [self.control_points[0], q01.point, q012.point],
                weights: [self.weights[0], q01.weight, q012.weight],
                start_phase: self.start_phase,
                end_phase: split_phase,
                direction: self.direction,
            },
            Self {
                ellipse: self.ellipse,
                control_points: [q012.point, q12.point, self.control_points[2]],
                weights: [q012.weight, q12.weight, self.weights[2]],
                start_phase: split_phase,
                end_phase: self.end_phase,
                direction: self.direction,
            },
        ]
    }

    #[inline]
    fn rational_levels(
        &self,
        t: SegmentParam<I>,
    ) -> (RationalControl<I>, RationalControl<I>, RationalControl<I>) {
        let p0 = RationalControl::new(self.control_points[0], self.weights[0]);
        let p1 = RationalControl::new(self.control_points[1], self.weights[1]);
        let p2 = RationalControl::new(self.control_points[2], self.weights[2]);
        let q01 = p0.mix(p1, t);
        let q12 = p1.mix(p2, t);
        let q012 = q01.mix(q12, t);

        (q01, q12, q012)
    }

    /// Checks the structural invariants available before arc math is enabled.
    ///
    /// Geometric agreement between the rational curve and `ellipse` is not
    /// asserted here because boolean overlay processing may snap endpoints.
    #[inline]
    pub(crate) fn debug_assert_invariants(&self) {
        self.ellipse.debug_assert_invariants();
        self.start_phase.debug_assert_invariants();
        self.end_phase.debug_assert_invariants();

        let denominator = FixedScale::<I>::DENOMINATOR;
        for weight in self.weights {
            let weight = weight.to_wide();
            debug_assert!(
                weight > I::Wide::ZERO && weight <= denominator,
                "arc rational weights must be in the fixed-point interval (0, 1]"
            );
        }

        debug_assert!(
            self.control_points[0] != self.control_points[2],
            "normalized arc must have a non-zero chord"
        );

        let start_cos = self.start_phase.cos.to_wide();
        let start_sin = self.start_phase.sin.to_wide();
        let end_cos = self.end_phase.cos.to_wide();
        let end_sin = self.end_phase.sin.to_wide();
        let cross = start_cos * end_sin - start_sin * end_cos;

        match self.direction {
            ArcDirection::Clockwise => {
                debug_assert!(
                    cross < I::Wide::ZERO,
                    "clockwise arc must span less than 180 degrees"
                );
            }
            ArcDirection::CounterClockwise => {
                debug_assert!(
                    cross > I::Wide::ZERO,
                    "counter-clockwise arc must span less than 180 degrees"
                );
            }
        }
    }
}

#[inline]
fn is_ordered<I: IntNumber>(a: I, b: I, c: I) -> bool {
    a <= b && b <= c || a >= b && b >= c
}

#[derive(Clone, Copy)]
struct RationalControl<I: IntNumber> {
    point: IntPoint<I>,
    weight: I,
}

impl<I: IntNumber> RationalControl<I> {
    #[inline]
    fn new(point: IntPoint<I>, weight: I) -> Self {
        Self { point, weight }
    }

    #[inline]
    fn mix(self, other: Self, t: SegmentParam<I>) -> Self {
        let denominator = SegmentParam::<I>::DENOMINATOR;
        let t = t.value();
        let one_minus_t = denominator - t;

        // Reduce the two homogeneous coefficients back to the stored fixed
        // scale before multiplying by coordinates. This keeps every product
        // inside `I::Wide`; the rounding policy is deterministic and matches
        // the integer de Casteljau steps used by polynomial segments.
        let a = FixedScale::<I>::div_round(one_minus_t * self.weight.to_wide(), denominator);
        let b = FixedScale::<I>::div_round(t * other.weight.to_wide(), denominator);
        let weight = a + b;
        debug_assert!(weight > I::Wide::ZERO);

        let x = FixedScale::<I>::div_round(a * self.point.x.to_wide() + b * other.point.x.to_wide(), weight);
        let y = FixedScale::<I>::div_round(a * self.point.y.to_wide() + b * other.point.y.to_wide(), weight);

        Self {
            point: IntPoint::new(I::from_wide(x), I::from_wide(y)),
            weight: I::from_wide(weight),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quarter_circle() -> ArcSegment<i32> {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let diagonal = 759_250_125;

        ArcSegment {
            ellipse: EllipseFrame {
                center: IntPoint::new(0, 0),
                axis_x: ArcVector { x: 100, y: 0 },
                axis_y: ArcVector { x: 0, y: 100 },
            },
            control_points: [
                IntPoint::new(100, 0),
                IntPoint::new(100, 100),
                IntPoint::new(0, 100),
            ],
            weights: [one, diagonal, one],
            start_phase: ArcPhase { cos: one, sin: 0 },
            end_phase: ArcPhase { cos: 0, sin: one },
            direction: ArcDirection::CounterClockwise,
        }
    }

    #[test]
    fn rational_point_at_preserves_endpoints_and_evaluates_midpoint() {
        let arc = quarter_circle();

        assert_eq!(arc.point_at(SegmentParam::new(0)), arc.control_points[0]);
        assert_eq!(
            arc.point_at(SegmentParam::new(SegmentParam::<i32>::DENOMINATOR as i32)),
            arc.control_points[2]
        );
        assert_eq!(arc.point_at(SegmentParam::half()), IntPoint::new(71, 71));
    }

    #[test]
    fn rational_split_shares_geometry_weights_and_phase() {
        let arc = quarter_circle();
        let [left, right] = arc.rational_split(SegmentParam::half());

        assert_eq!(left.control_points[2], IntPoint::new(71, 71));
        assert_eq!(right.control_points[0], left.control_points[2]);
        assert_eq!(left.weights[2], right.weights[0]);
        assert_eq!(left.end_phase, right.start_phase);
        assert_eq!(left.end_phase.cos, 759_250_125);
        assert_eq!(left.end_phase.sin, 759_250_125);
        left.debug_assert_invariants();
        right.debug_assert_invariants();
    }

    #[test]
    fn reverse_is_an_involution() {
        let arc = quarter_circle();
        let reversed = arc.reversed();

        assert_eq!(
            reversed.control_points,
            [
                arc.control_points[2],
                arc.control_points[1],
                arc.control_points[0],
            ]
        );
        assert_eq!(reversed.weights, [arc.weights[2], arc.weights[1], arc.weights[0]]);
        assert_eq!(reversed.start_phase, arc.end_phase);
        assert_eq!(reversed.end_phase, arc.start_phase);
        assert_eq!(reversed.direction, ArcDirection::Clockwise);
        assert_eq!(reversed.reversed(), arc);
    }

    #[test]
    fn phase_at_supports_rotated_ellipse_frame() {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let ellipse = EllipseFrame {
            center: IntPoint::new(10, -20),
            axis_x: ArcVector { x: 80, y: 60 },
            axis_y: ArcVector { x: -30, y: 40 },
        };

        assert_eq!(
            ellipse.phase_at(IntPoint::new(90, 40)),
            Some(ArcPhase { cos: one, sin: 0 })
        );
        assert_eq!(
            ellipse.phase_at(IntPoint::new(-20, 20)),
            Some(ArcPhase { cos: 0, sin: one })
        );
    }
}
