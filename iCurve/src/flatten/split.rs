use crate::flatten::segment::{CubicSegment, NormalizedSegment, QuadSegment};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

pub trait SplitAt<T: FloatNumber> {
    type Output;

    fn split_at(&self, t: T) -> Self::Output;

    fn split_at_half(&self) -> Self::Output {
        self.split_at(T::from_float(0.5))
    }
}

impl<P: FloatPointCompatible> SplitAt<P::Scalar> for NormalizedSegment<P> {
    type Output = [Self; 2];

    fn split_at(&self, t: P::Scalar) -> Self::Output {
        match self {
            Self::Line(_) => panic!("Line segment split is not supported"),
            Self::Arc(_) => panic!("Arc segment split is not supported"),
            Self::Quad(segment) => {
                let [a, b] = segment.split_at(t);
                [Self::Quad(a), Self::Quad(b)]
            }
            Self::Cubic(segment) => {
                let [a, b] = segment.split_at(t);
                [Self::Cubic(a), Self::Cubic(b)]
            }
        }
    }
}

impl<P: FloatPointCompatible> SplitAt<P::Scalar> for QuadSegment<P> {
    type Output = [Self; 2];

    fn split_at(&self, t: P::Scalar) -> Self::Output {
        let [p0, p1, p2] = self.control_points;

        let p01 = point_at(p0, p1, t);
        let p12 = point_at(p1, p2, t);
        let p012 = point_at(p01, p12, t);

        [
            Self {
                control_points: [p0, p01, p012],
            },
            Self {
                control_points: [p012, p12, p2],
            },
        ]
    }
}

impl<P: FloatPointCompatible> SplitAt<P::Scalar> for CubicSegment<P> {
    type Output = [Self; 2];

    fn split_at(&self, t: P::Scalar) -> Self::Output {
        let [p0, p1, p2, p3] = self.control_points;
        let p01 = point_at(p0, p1, t);
        let p12 = point_at(p1, p2, t);
        let p23 = point_at(p2, p3, t);
        let p012 = point_at(p01, p12, t);
        let p123 = point_at(p12, p23, t);
        let p0123 = point_at(p012, p123, t);

        [
            Self {
                control_points: [p0, p01, p012, p0123],
            },
            Self {
                control_points: [p0123, p123, p23, p3],
            },
        ]
    }
}

fn point_at<P: FloatPointCompatible>(a: P, b: P, t: P::Scalar) -> P {
    P::from_xy(a.x() + (b.x() - a.x()) * t, a.y() + (b.y() - a.y()) * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::segment::{ArcSegment, LineSegment};

    #[test]
    fn split_quad_at_half() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 2.0], [4.0, 0.0]],
        };

        let [a, b] = segment.split_at(0.5);

        assert_eq!(a.control_points, [[0.0, 0.0], [1.0, 1.0], [2.0, 1.0]]);
        assert_eq!(b.control_points, [[2.0, 1.0], [3.0, 1.0], [4.0, 0.0]]);
    }

    #[test]
    fn split_quad_at_custom_parameter() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 2.0], [4.0, 0.0]],
        };

        let [a, b] = segment.split_at(0.25);

        assert_eq!(a.control_points, [[0.0, 0.0], [0.5, 0.5], [1.0, 0.75]]);
        assert_eq!(b.control_points, [[1.0, 0.75], [2.5, 1.5], [4.0, 0.0]]);
    }

    #[test]
    fn split_cubic_at_half() {
        let segment = CubicSegment {
            control_points: [[0.0, 0.0], [2.0, 3.0], [4.0, 3.0], [6.0, 0.0]],
        };

        let [a, b] = segment.split_at(0.5);

        assert_eq!(
            a.control_points,
            [[0.0, 0.0], [1.0, 1.5], [2.0, 2.25], [3.0, 2.25]]
        );
        assert_eq!(
            b.control_points,
            [[3.0, 2.25], [4.0, 2.25], [5.0, 1.5], [6.0, 0.0]]
        );
    }

    #[test]
    fn split_segment_at_half() {
        let segment = NormalizedSegment::Quad(QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 2.0], [4.0, 0.0]],
        });

        let [a, b] = segment.split_at(0.5);

        match (a, b) {
            (NormalizedSegment::Quad(a), NormalizedSegment::Quad(b)) => {
                assert_eq!(a.control_points, [[0.0, 0.0], [1.0, 1.0], [2.0, 1.0]]);
                assert_eq!(b.control_points, [[2.0, 1.0], [3.0, 1.0], [4.0, 0.0]]);
            }
            _ => panic!("Expected quad segments"),
        }
    }

    #[test]
    #[should_panic(expected = "Line segment split is not supported")]
    fn split_segment_line_panics() {
        let line = NormalizedSegment::Line(LineSegment {
            control_points: [[0.0, 0.0], [1.0, 1.0]],
        });

        line.split_at(0.5);
    }

    #[test]
    #[should_panic(expected = "Arc segment split is not supported")]
    fn split_segment_arc_panics() {
        let arc = NormalizedSegment::Arc(ArcSegment {
            p0: [1.0, 0.0],
            p1: [0.0, 1.0],
            center: [0.0, 0.0],
            radii: [1.0, 1.0],
            rotation: 0.0,
            start_angle: 0.0,
            sweep_angle: core::f64::consts::FRAC_PI_2,
        });

        arc.split_at(0.5);
    }
}
