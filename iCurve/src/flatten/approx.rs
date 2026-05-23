use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

use crate::flatten::segment::{CubicSegment, QuadSegment, SegmentKind};

#[derive(Clone, Copy)]
pub struct LineApproximation<T: FloatNumber> {
    pub min_cos: T,
    pub min_segment_sqr_length: T,
}

pub trait LineApproximationSplit<T: FloatNumber> {
    fn is_split_required(&self, approximation: LineApproximation<T>) -> bool;
}

impl<P: FloatPointCompatible> LineApproximationSplit<P::Scalar> for SegmentKind<P> {
    fn is_split_required(&self, approximation: LineApproximation<P::Scalar>) -> bool {
        match self {
            Self::Line(_) => false,
            Self::Arc(_) => panic!("Arc segment approximation is not supported"),
            Self::Quad(segment) => segment.is_split_required(approximation),
            Self::Cubic(segment) => segment.is_split_required(approximation),
        }
    }
}

impl<P: FloatPointCompatible> LineApproximationSplit<P::Scalar> for QuadSegment<P> {
    fn is_split_required(&self, approximation: LineApproximation<P::Scalar>) -> bool {
        let [p0, p1, p2] = self.control_points;
        let chord = vector(p0, p2);

        if sqr_length(chord) <= approximation.min_segment_sqr_length {
            return false;
        }

        !is_angle_accepted(chord, vector(p0, p1), approximation.min_cos)
            || !is_angle_accepted(chord, vector(p1, p2), approximation.min_cos)
    }
}

impl<P: FloatPointCompatible> LineApproximationSplit<P::Scalar> for CubicSegment<P> {
    fn is_split_required(&self, approximation: LineApproximation<P::Scalar>) -> bool {
        let [p0, p1, p2, p3] = self.control_points;
        let chord = vector(p0, p3);

        if sqr_length(chord) <= approximation.min_segment_sqr_length {
            return false;
        }

        !is_angle_accepted(chord, vector(p0, p1), approximation.min_cos)
            || !is_angle_accepted(chord, vector(p2, p3), approximation.min_cos)
    }
}

fn is_angle_accepted<P: FloatPointCompatible>(chord: P, derivative: P, min_cos: P::Scalar) -> bool {
    let dot = dot_product(chord, derivative);
    dot >= P::Scalar::from_float(0.0)
        && dot * dot >= min_cos * min_cos * sqr_length(chord) * sqr_length(derivative)
}

fn vector<P: FloatPointCompatible>(a: P, b: P) -> P {
    P::from_xy(b.x() - a.x(), b.y() - a.y())
}

fn dot_product<P: FloatPointCompatible>(a: P, b: P) -> P::Scalar {
    a.x() * b.x() + a.y() * b.y()
}

fn sqr_length<P: FloatPointCompatible>(p: P) -> P::Scalar {
    dot_product(p, p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::segment::{ArcSegment, LineSegment};

    fn approximation() -> LineApproximation<f64> {
        LineApproximation {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0001,
        }
    }

    #[test]
    fn quad_split_not_required_for_flat_segment() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 0.0], [4.0, 0.0]],
        };

        assert!(!segment.is_split_required(approximation()));
    }

    #[test]
    fn quad_split_required_for_sharp_tangent() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0], [0.0, 2.0], [4.0, 0.0]],
        };

        assert!(segment.is_split_required(approximation()));
    }

    #[test]
    fn cubic_split_not_required_for_flat_segment() {
        let segment = CubicSegment {
            control_points: [[0.0, 0.0], [2.0, 0.0], [4.0, 0.0], [6.0, 0.0]],
        };

        assert!(!segment.is_split_required(approximation()));
    }

    #[test]
    fn cubic_split_required_for_sharp_tangent() {
        let segment = CubicSegment {
            control_points: [[0.0, 0.0], [0.0, 2.0], [4.0, 2.0], [6.0, 0.0]],
        };

        assert!(segment.is_split_required(approximation()));
    }

    #[test]
    fn split_not_required_for_short_segment() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0], [0.0, 2.0], [0.01, 0.0]],
        };

        assert!(!segment.is_split_required(LineApproximation {
            min_cos: 0.99,
            min_segment_sqr_length: 0.0002,
        }));
    }

    #[test]
    fn segment_kind_line_split_not_required() {
        let segment = SegmentKind::Line(LineSegment {
            control_points: [[0.0, 0.0], [1.0, 0.0]],
        });

        assert!(!segment.is_split_required(approximation()));
    }

    #[test]
    #[should_panic(expected = "Arc segment approximation is not supported")]
    fn segment_kind_arc_panics() {
        let segment = SegmentKind::Arc(ArcSegment {
            p0: [1.0, 0.0],
            p1: [0.0, 1.0],
            center: [0.0, 0.0],
            radii: [1.0, 1.0],
            rotation: 0.0,
            start_angle: 0.0,
            sweep_angle: core::f64::consts::FRAC_PI_2,
        });

        segment.is_split_required(approximation());
    }
}
