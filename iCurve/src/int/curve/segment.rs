use crate::kernel::int::curve::cubic::CubicSegment;
use crate::kernel::int::curve::line::LineSegment;
use crate::kernel::int::curve::quad::QuadSegment;
use crate::kernel::int::curve::segment::Segment;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

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
}

impl<I: IntNumber> CurveSegment<I> {
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
        }
    }
}
