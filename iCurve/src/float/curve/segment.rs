use crate::float::curve::arc::RationalArc;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

/// Segment of a float [`CurvePath`](crate::float::CurvePath).
///
/// The containing path supplies the start point; each variant stores its
/// remaining control data and endpoint.
#[derive(Clone, PartialEq)]
pub enum CurveSegment<P: FloatPointCompatible> {
    /// Straight segment.
    Line {
        /// Endpoint.
        to: P,
    },
    /// Quadratic Bézier segment.
    Quad {
        /// Quadratic control point.
        ctrl: P,
        /// Endpoint.
        to: P,
    },
    /// Cubic Bézier segment.
    Cubic {
        /// First cubic control point.
        ctrl0: P,
        /// Second cubic control point.
        ctrl1: P,
        /// Endpoint.
        to: P,
    },
    /// Authoritative rational quadratic elliptic arc.
    Arc {
        /// Rational arc data, including its own start and end points.
        arc: RationalArc<P>,
    },
}

impl<P> core::fmt::Debug for CurveSegment<P>
where
    P: FloatPointCompatible + core::fmt::Debug,
    P::Scalar: core::fmt::Debug,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Line { to } => formatter.debug_struct("Line").field("to", to).finish(),
            Self::Quad { ctrl, to } => formatter
                .debug_struct("Quad")
                .field("ctrl", ctrl)
                .field("to", to)
                .finish(),
            Self::Cubic { ctrl0, ctrl1, to } => formatter
                .debug_struct("Cubic")
                .field("ctrl0", ctrl0)
                .field("ctrl1", ctrl1)
                .field("to", to)
                .finish(),
            Self::Arc { arc } => formatter.debug_struct("Arc").field("arc", arc).finish(),
        }
    }
}

impl<P: FloatPointCompatible> CurveSegment<P> {
    /// Returns this segment's endpoint.
    #[inline]
    pub fn end_point(&self) -> P {
        match self {
            Self::Line { to } | Self::Quad { to, .. } | Self::Cubic { to, .. } => *to,
            Self::Arc { arc } => arc.end_point(),
        }
    }
}
