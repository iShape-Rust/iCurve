use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use crate::flatten::segment::NormalizedSegment;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::quad::QuadSegment;

#[derive(Clone, Copy)]
pub struct LineApproximation<T: FloatNumber> {
    pub min_cos: T,
    pub min_segment_sqr_length: T,
}

pub trait LineApproximationSplit<T: FloatNumber> {
    fn is_split_required(&self, approximation: LineApproximation<T>) -> bool;
}

impl<T: FloatNumber> LineApproximationSplit<T> for NormalizedSegment<T> {
    fn is_split_required(&self, approximation: LineApproximation<T>) -> bool {
        match self {
            Self::Line(_) => false,
            Self::Quad(segment) => segment.is_split_required(approximation),
            Self::Cubic(segment) => segment.is_split_required(approximation),
        }
    }
}

impl<T: FloatNumber> LineApproximationSplit<T> for QuadSegment<T> {
    fn is_split_required(&self, approximation: LineApproximation<T>) -> bool {
        let [p0, p1, p2] = self.control_points;
        let chord = p0 - p2;

        if chord.sqr_length() <= approximation.min_segment_sqr_length {
            return false;
        }

        !is_angle_accepted(chord, p0 - p1, approximation.min_cos)
            || !is_angle_accepted(chord, p1 - p2, approximation.min_cos)
    }
}

impl<T: FloatNumber> LineApproximationSplit<T> for CubicSegment<T> {
    fn is_split_required(&self, approximation: LineApproximation<T>) -> bool {
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
    use crate::kernel::curve::line::LineSegment;
    use super::*;

    fn approximation() -> LineApproximation<f64> {
        LineApproximation {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0001,
        }
    }

    #[test]
    fn quad_split_not_required_for_flat_segment() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0].into(), [2.0, 0.0].into(), [4.0, 0.0].into()],
        };

        assert!(!segment.is_split_required(approximation()));
    }

    #[test]
    fn quad_split_required_for_sharp_tangent() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0].into(), [0.0, 2.0].into(), [4.0, 0.0].into()],
        };

        assert!(segment.is_split_required(approximation()));
    }

    #[test]
    fn cubic_split_not_required_for_flat_segment() {
        let segment = CubicSegment {
            control_points: [[0.0, 0.0].into(), [2.0, 0.0].into(), [4.0, 0.0].into(), [6.0, 0.0].into()],
        };

        assert!(!segment.is_split_required(approximation()));
    }

    #[test]
    fn cubic_split_required_for_sharp_tangent() {
        let segment = CubicSegment {
            control_points: [[0.0, 0.0].into(), [0.0, 2.0].into(), [4.0, 2.0].into(), [6.0, 0.0].into()],
        };

        assert!(segment.is_split_required(approximation()));
    }

    #[test]
    fn split_not_required_for_short_segment() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0].into(), [0.0, 2.0].into(), [0.01, 0.0].into()],
        };

        assert!(!segment.is_split_required(LineApproximation {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0002,
        }));
    }

    #[test]
    fn segment_line_split_not_required() {
        let segment = NormalizedSegment::Line(LineSegment {
            control_points: [[0.0, 0.0].into(), [1.0, 0.0].into()],
        });

        assert!(!segment.is_split_required(approximation()));
    }
}
