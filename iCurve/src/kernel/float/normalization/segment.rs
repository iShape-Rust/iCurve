use crate::float::curve::segment::CurveSegment;
use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::line::FloatLineSegment;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use crate::kernel::float::curve::segment::FloatSegment;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;

pub(crate) trait CurveSegmentNormalization<P: FloatPointCompatible, I: IntNumber> {
    fn try_normalize(
        &self,
        p0: FloatPoint<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<FloatSegment<P::Scalar>>,
    ) -> Result<P, FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible, I: IntNumber> CurveSegmentNormalization<P, I> for CurveSegment<P> {
    fn try_normalize(
        &self,
        p0: FloatPoint<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<FloatSegment<P::Scalar>>,
    ) -> Result<P, FloatPointAdapterRangeError> {
        match *self {
            CurveSegment::Line { to } => {
                let p1 = FloatPoint::from_point(to);
                let line = FloatLineSegment {
                    control_points: [p0, p1],
                };
                if let Some(segment) = line.try_with_adapter(adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Quad { ctrl, to } => {
                let p1 = FloatPoint::from_point(ctrl);
                let p2 = FloatPoint::from_point(to);
                let quad = FloatQuadSegment {
                    control_points: [p0, p1, p2],
                };
                if let Some(segment) = quad.try_with_adapter(adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                let p1 = FloatPoint::from_point(ctrl0);
                let p2 = FloatPoint::from_point(ctrl1);
                let p3 = FloatPoint::from_point(to);

                let cubic = FloatCubicSegment {
                    control_points: [p0, p1, p2, p3],
                };
                let segments = cubic.try_with_adapter(adapter)?;
                if !segments.is_empty() {
                    output.extend_from_slice(segments.as_slice())
                }
                Ok(to)
            }
        }
    }
}
