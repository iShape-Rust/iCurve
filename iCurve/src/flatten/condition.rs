use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use crate::kernel::float::curve::segment::FloatSegment;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Clone, Copy)]
pub struct FlatParams<T: FloatNumber> {
    pub min_cos: T,
    pub min_segment_sqr_length: T,
}

pub trait FlatCondition<T: FloatNumber> {
    fn is_not_flat(&self, params: FlatParams<T>) -> bool;
}

impl<T: FloatNumber> FlatCondition<T> for FloatSegment<T> {
    fn is_not_flat(&self, params: FlatParams<T>) -> bool {
        match self {
            Self::Line(_) => false,
            Self::Quad(segment) => segment.is_not_flat(params),
            Self::Cubic(segment) => segment.is_not_flat(params),
        }
    }
}

impl<T: FloatNumber> FlatCondition<T> for FloatQuadSegment<T> {
    fn is_not_flat(&self, params: FlatParams<T>) -> bool {
        let [p0, p1, p2] = self.control_points;
        let chord = p0 - p2;

        if chord.sqr_length() <= params.min_segment_sqr_length {
            return false;
        }

        !is_angle_accepted(chord, p0 - p1, params.min_cos)
            || !is_angle_accepted(chord, p1 - p2, params.min_cos)
    }
}

impl<T: FloatNumber> FlatCondition<T> for FloatCubicSegment<T> {
    fn is_not_flat(&self, approximation: FlatParams<T>) -> bool {
        let [p0, p1, p2, p3] = self.control_points;
        let chord = p0 - p3;

        if chord.sqr_length() <= approximation.min_segment_sqr_length {
            return false;
        }

        !is_angle_accepted(chord, p0 - p1, approximation.min_cos)
            || !is_angle_accepted(chord, p2 - p3, approximation.min_cos)
    }
}

fn is_angle_accepted<T: FloatNumber>(chord: FloatPoint<T>, derivative: FloatPoint<T>, min_cos: T) -> bool {
    let dot = chord.dot_product(derivative);
    dot >= T::ZERO && dot * dot >= min_cos * min_cos * chord.sqr_length() * derivative.sqr_length()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::float::curve::line::FloatLineSegment;

    fn approximation() -> FlatParams<f64> {
        FlatParams {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0001,
        }
    }

    #[test]
    fn quad_split_not_required_for_flat_segment() {
        let segment = FloatQuadSegment {
            control_points: [[0.0, 0.0].into(), [2.0, 0.0].into(), [4.0, 0.0].into()],
        };

        assert!(!segment.is_not_flat(approximation()));
    }

    #[test]
    fn quad_split_required_for_sharp_tangent() {
        let segment = FloatQuadSegment {
            control_points: [[0.0, 0.0].into(), [0.0, 2.0].into(), [4.0, 0.0].into()],
        };

        assert!(segment.is_not_flat(approximation()));
    }

    #[test]
    fn cubic_split_not_required_for_flat_segment() {
        let segment = FloatCubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [2.0, 0.0].into(),
                [4.0, 0.0].into(),
                [6.0, 0.0].into(),
            ],
        };

        assert!(!segment.is_not_flat(approximation()));
    }

    #[test]
    fn cubic_split_required_for_sharp_tangent() {
        let segment = FloatCubicSegment {
            control_points: [
                [0.0, 0.0].into(),
                [0.0, 2.0].into(),
                [4.0, 2.0].into(),
                [6.0, 0.0].into(),
            ],
        };

        assert!(segment.is_not_flat(approximation()));
    }

    #[test]
    fn split_not_required_for_short_segment() {
        let segment = FloatQuadSegment {
            control_points: [[0.0, 0.0].into(), [0.0, 2.0].into(), [0.01, 0.0].into()],
        };

        assert!(!segment.is_not_flat(FlatParams {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0002,
        }));
    }

    #[test]
    fn segment_line_split_not_required() {
        let segment = FloatSegment::Line(FloatLineSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 0.0].into()],
        });

        assert!(!segment.is_not_flat(approximation()));
    }
}
