use crate::int::CURVE_COORDINATE_SAFETY_BITS;
use i_overlay::core::solver::Precision;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

pub(super) struct SnapRadius {
    current: usize,
    progression: usize,
}

impl SnapRadius {
    #[inline]
    pub(super) fn new<I: IntNumber>(precision: Precision) -> Self {
        Self {
            current: precision.start,
            progression: Self::scaled_progression::<I>(precision.progression),
        }
    }

    #[inline]
    pub(super) fn increment(&mut self) {
        self.current = self.current.saturating_add(self.progression);
    }

    #[inline]
    pub(super) fn radius<I: IntNumber>(&self) -> I::Wide {
        // The radius is a squared distance. Capping its exponent at twice the
        // safe coordinate magnitude prevents snapping beyond iCurve's input
        // domain and keeps the shift valid for every supported wide integer.
        let coordinate_bits = I::BITS - CURVE_COORDINATE_SAFETY_BITS;
        let max_exponent = (2 * coordinate_bits) as usize;
        I::Wide::ONE << self.current.min(max_exponent) as u32
    }

    #[inline]
    fn scaled_progression<I: IntNumber>(progression: usize) -> usize {
        if progression == 0 {
            return 0;
        }

        // Precision progression was tuned for i32. Scale its exponent step so
        // wider solvers do not require proportionally more refinement rounds.
        let numerator = progression.saturating_mul(I::BITS as usize);
        let quotient = numerator / i32::BITS as usize;
        let remainder = numerator % i32::BITS as usize;
        quotient.saturating_add(usize::from(remainder != 0)).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_precision_progression() {
        let mut radius = SnapRadius::new::<i32>(Precision::MEDIUM);

        assert_eq!(radius.radius::<i32>(), 1);
        radius.increment();
        assert_eq!(radius.radius::<i32>(), 4);
        radius.increment();
        assert_eq!(radius.radius::<i32>(), 16);
    }

    #[test]
    fn scales_progression_from_i32_solver_width() {
        assert_eq!(SnapRadius::scaled_progression::<i16>(1), 1);
        assert_eq!(SnapRadius::scaled_progression::<i16>(2), 1);
        assert_eq!(SnapRadius::scaled_progression::<i16>(3), 2);

        assert_eq!(SnapRadius::scaled_progression::<i32>(1), 1);
        assert_eq!(SnapRadius::scaled_progression::<i32>(2), 2);
        assert_eq!(SnapRadius::scaled_progression::<i32>(3), 3);

        assert_eq!(SnapRadius::scaled_progression::<i64>(1), 2);
        assert_eq!(SnapRadius::scaled_progression::<i64>(2), 4);
        assert_eq!(SnapRadius::scaled_progression::<i64>(3), 6);
        assert_eq!(SnapRadius::scaled_progression::<i64>(0), 0);
    }

    #[test]
    fn keeps_initial_radius_independent_of_solver_width() {
        let precision = Precision {
            start: 2,
            progression: 1,
        };

        assert_eq!(SnapRadius::new::<i16>(precision).radius::<i16>(), 4);
        assert_eq!(SnapRadius::new::<i32>(precision).radius::<i32>(), 4);
        assert_eq!(SnapRadius::new::<i64>(precision).radius::<i64>(), 4);
    }

    #[test]
    fn caps_exponent_for_each_integer_width() {
        let radius = SnapRadius::new::<i32>(Precision {
            start: usize::MAX,
            progression: usize::MAX,
        });

        assert_eq!(radius.radius::<i16>(), 1_i32 << 20);
        assert_eq!(radius.radius::<i32>(), 1_i64 << 52);
        assert_eq!(radius.radius::<i64>(), 1_i128 << 116);
    }
}
