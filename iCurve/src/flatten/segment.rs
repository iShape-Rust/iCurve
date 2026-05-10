use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

pub(crate) struct Segment<P: FloatPointCompatible> {
    pub(crate) segment_kind: SegmentKind<P>,
    pub(crate) shape_type: ShapeType,
}

pub(crate) enum SegmentKind<P: FloatPointCompatible> {
    Line(LineSegment<P>),
    Quad(QuadSegment<P>),
    Cubic(CubicSegment<P>),
    Arc(ArcSegment<P>),
}

impl<P: FloatPointCompatible> SegmentKind<P> {
    pub(crate) fn split_at_half(&self) -> Option<[Self; 2]> {
        match self {
            Self::Line(_) | Self::Arc(_) => None,
            Self::Quad(segment) => {
                let [a, b] = segment.split_at_half();
                Some([Self::Quad(a), Self::Quad(b)])
            }
            Self::Cubic(segment) => {
                let [a, b] = segment.split_at_half();
                Some([Self::Cubic(a), Self::Cubic(b)])
            }
        }
    }
}

pub(crate) struct SubSegment<T: FloatNumber> {
    pub(crate) segment_index: usize,
    pub(crate) t0: T,
    pub(crate) t1: T,
}
pub(crate) struct LineSegment<P: FloatPointCompatible> {
    pub(crate) control_points: [P; 2],
}

pub(crate) struct QuadSegment<P: FloatPointCompatible> {
    pub(crate) control_points: [P; 3],
}

impl<P: FloatPointCompatible> QuadSegment<P> {
    pub(crate) fn split_at_half(&self) -> [Self; 2] {
        let [p0, p1, p2] = self.control_points;

        let p01 = mid_point(p0, p1);
        let p12 = mid_point(p1, p2);
        let p012 = mid_point(p01, p12);

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

pub(crate) struct CubicSegment<P: FloatPointCompatible> {
    pub(crate) control_points: [P; 4],
}

impl<P: FloatPointCompatible> CubicSegment<P> {
    pub(crate) fn split_at_half(&self) -> [Self; 2] {
        let [p0, p1, p2, p3] = self.control_points;

        let p01 = mid_point(p0, p1);
        let p12 = mid_point(p1, p2);
        let p23 = mid_point(p2, p3);
        let p012 = mid_point(p01, p12);
        let p123 = mid_point(p12, p23);
        let p0123 = mid_point(p012, p123);

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

pub(crate) struct ArcSegment<P: FloatPointCompatible> {
    pub(crate) p0: P,
    pub(crate) p1: P,
    pub(crate) center: P,
    pub(crate) radii: P,
    pub(crate) rotation: P::Scalar,
    pub(crate) start_angle: P::Scalar,
    pub(crate) sweep_angle: P::Scalar,
}

fn mid_point<P: FloatPointCompatible>(a: P, b: P) -> P {
    let half = P::Scalar::from_float(0.5);
    P::from_xy((a.x() + b.x()) * half, (a.y() + b.y()) * half)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_quad_at_half() {
        let segment = QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 2.0], [4.0, 0.0]],
        };

        let [a, b] = segment.split_at_half();

        assert_eq!(a.control_points, [[0.0, 0.0], [1.0, 1.0], [2.0, 1.0]]);
        assert_eq!(b.control_points, [[2.0, 1.0], [3.0, 1.0], [4.0, 0.0]]);
    }

    #[test]
    fn split_cubic_at_half() {
        let segment = CubicSegment {
            control_points: [[0.0, 0.0], [2.0, 3.0], [4.0, 3.0], [6.0, 0.0]],
        };

        let [a, b] = segment.split_at_half();

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
    fn split_segment_kind_at_half() {
        let segment = SegmentKind::Quad(QuadSegment {
            control_points: [[0.0, 0.0], [2.0, 2.0], [4.0, 0.0]],
        });

        let [a, b] = segment.split_at_half().expect("quad should split");

        match (a, b) {
            (SegmentKind::Quad(a), SegmentKind::Quad(b)) => {
                assert_eq!(a.control_points, [[0.0, 0.0], [1.0, 1.0], [2.0, 1.0]]);
                assert_eq!(b.control_points, [[2.0, 1.0], [3.0, 1.0], [4.0, 0.0]]);
            }
            _ => panic!("Expected quad segments"),
        }
    }

    #[test]
    fn split_segment_kind_unsupported_segments() {
        let line = SegmentKind::Line(LineSegment {
            control_points: [[0.0, 0.0], [1.0, 1.0]],
        });
        let arc = SegmentKind::Arc(ArcSegment {
            p0: [1.0, 0.0],
            p1: [0.0, 1.0],
            center: [0.0, 0.0],
            radii: [1.0, 1.0],
            rotation: 0.0,
            start_angle: 0.0,
            sweep_angle: core::f64::consts::FRAC_PI_2,
        });

        assert!(line.split_at_half().is_none());
        assert!(arc.split_at_half().is_none());
    }
}
