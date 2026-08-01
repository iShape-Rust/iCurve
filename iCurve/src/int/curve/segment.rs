use crate::int::arc::RationalArc;
use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurveSegment<I: IntNumber> {
    Line {
        to: IntPoint<I>,
    },
    Quad {
        ctrl: IntPoint<I>,
        to: IntPoint<I>,
    },
    Cubic {
        ctrl0: IntPoint<I>,
        ctrl1: IntPoint<I>,
        to: IntPoint<I>,
    },
    Arc {
        arc: RationalArc<I>,
    },
}

impl<I: IntNumber> CurveSegment<I> {
    /// Returns this segment's endpoint.
    pub fn end_point(&self) -> IntPoint<I> {
        match self {
            Self::Line { to } | Self::Quad { to, .. } | Self::Cubic { to, .. } => *to,
            Self::Arc { arc } => arc.control_points[2],
        }
    }

    pub(crate) fn from_kernel_segment(segment: Segment<I>) -> Self {
        match segment {
            Segment::Line(line) => Self::Line {
                to: line.control_points[1],
            },
            Segment::Quad(quad) => Self::Quad {
                ctrl: quad.control_points[1],
                to: quad.control_points[2],
            },
            Segment::Cubic(cubic) => Self::Cubic {
                ctrl0: cubic.control_points[1],
                ctrl1: cubic.control_points[2],
                to: cubic.control_points[3],
            },
            Segment::Arc(arc) => Self::Arc { arc },
        }
    }

    pub(crate) fn into_kernel_segment(self, start: IntPoint<I>) -> (Segment<I>, IntPoint<I>) {
        match self {
            Self::Line { to } => (
                Segment::Line(LineSegment {
                    control_points: [start, to],
                }),
                to,
            ),
            Self::Quad { ctrl, to } => (
                Segment::Quad(QuadSegment {
                    control_points: [start, ctrl, to],
                }),
                to,
            ),
            Self::Cubic { ctrl0, ctrl1, to } => (
                Segment::Cubic(CubicSegment {
                    control_points: [start, ctrl0, ctrl1, to],
                }),
                to,
            ),
            Self::Arc { arc } => {
                debug_assert!(
                    arc.control_points[0] == start,
                    "arc start must match its containing contour"
                );
                let end = arc.control_points[2];
                (Segment::Arc(arc), end)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::int::curve::arc::{ArcDirection, ArcPhase, ArcVector, EllipseFrame};
    use i_overlay::i_float::int::number::fixed_scale::FixedScale;

    #[test]
    fn arc_round_trips_through_kernel_segment() {
        let one = FixedScale::<i32>::DENOMINATOR as i32;
        let arc = RationalArc {
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
            weights: [one, 759_250_125, one],
            start_phase: ArcPhase { cos: one, sin: 0 },
            end_phase: ArcPhase { cos: 0, sin: one },
            direction: ArcDirection::CounterClockwise,
        };

        let public = CurveSegment::from_kernel_segment(Segment::Arc(arc));
        let (kernel, end) = public.into_kernel_segment(arc.control_points[0]);

        assert_eq!(end, arc.control_points[2]);
        match kernel {
            Segment::Arc(round_trip) => assert_eq!(round_trip, arc),
            _ => panic!("expected arc segment"),
        }
    }
}
