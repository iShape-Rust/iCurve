use core::cmp::Ordering;
use i_overlay::i_float::float::number::FloatNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SegmentParam<T: FloatNumber> {
    Start,
    Inner(T),
    End,
}

impl<T: FloatNumber> SegmentParam<T> {
    #[inline(always)]
    pub(crate) fn new(t: T) -> Self {
        if t == T::from_float(0.0) {
            Self::Start
        } else if t == T::from_float(1.0) {
            Self::End
        } else {
            Self::Inner(t)
        }
    }

    #[inline(always)]
    pub(crate) fn value(self) -> T {
        match self {
            Self::Start => T::from_float(0.0),
            Self::Inner(t) => t,
            Self::End => T::from_float(1.0),
        }
    }

    #[inline(always)]
    pub(crate) fn compare_with_epsilon(self, other: Self, epsilon: T) -> bool {
        (self.value() - other.value()).abs() < epsilon
    }

    #[inline(always)]
    pub(crate) fn is_in_unit_range(t: T, epsilon: T) -> bool {
        t >= -epsilon && t <= T::ONE + epsilon
    }

    #[inline(always)]
    pub(crate) fn clamp_unit(t: T) -> T {
        if t <= T::ZERO {
            T::ZERO
        } else if t >= T::ONE {
            T::ONE
        } else {
            t
        }
    }
}

impl<T: FloatNumber> Eq for SegmentParam<T> {}

impl<T: FloatNumber> PartialOrd for SegmentParam<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: FloatNumber> Ord for SegmentParam<T> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        use SegmentParam::*;

        match (*self, *other) {
            (Start, Start) => Ordering::Equal,
            (Start, _) => Ordering::Less,

            (End, End) => Ordering::Equal,
            (End, _) => Ordering::Greater,

            (Inner(_), Start) => Ordering::Greater,
            (Inner(_), End) => Ordering::Less,

            (Inner(a), Inner(b)) => a
                .partial_cmp(&b)
                .expect("SegmentParam::Inner must not contain NaN"),
        }
    }
}

impl<T: FloatNumber> From<T> for SegmentParam<T> {
    #[inline]
    fn from(t: T) -> Self {
        if t <= T::ZERO {
            SegmentParam::Start
        } else if t >= T::ONE {
            SegmentParam::End
        } else {
            SegmentParam::Inner(t)
        }
    }
}
