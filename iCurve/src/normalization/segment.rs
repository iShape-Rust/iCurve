use crate::curve::segment::CurveSegment;
use crate::kernel::curve::cubic::CubicSegment;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;
use crate::kernel::curve::split_at::SplitAt;
use alloc::vec::Vec;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;
use crate::normalization::cubic::CubicSelfIntersection;

pub(crate) trait CurveSegmentNormalization<P: FloatPointCompatible, I: IntNumber> {
    fn try_normalize(
        &self,
        p0: FloatPoint<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<Segment<P::Scalar>>,
    ) -> Result<P, FloatPointAdapterRangeError>;
}

impl<P: FloatPointCompatible, I: IntNumber> CurveSegmentNormalization<P, I> for CurveSegment<P> {
    fn try_normalize(
        &self,
        p0: FloatPoint<P::Scalar>,
        adapter: &FloatPointAdapter<FloatPoint<P::Scalar>, I>,
        output: &mut Vec<Segment<P::Scalar>>,
    ) -> Result<P, FloatPointAdapterRangeError> {
        match *self {
            CurveSegment::Line { to } => {
                let p1 = FloatPoint::from_point(to);
                if let Some(segment) = Segment::try_line_with_adapter(p0, p1, adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Quad { ctrl, to } => {
                let p1 = FloatPoint::from_point(ctrl);
                let p2 = FloatPoint::from_point(to);
                if let Some(segment) = Segment::try_quad_with_adapter(p0, p1, p2, adapter)? {
                    output.push(segment);
                };
                Ok(to)
            }
            CurveSegment::Cubic { ctrl0, ctrl1, to } => {
                let p1 = FloatPoint::from_point(ctrl0);
                let p2 = FloatPoint::from_point(ctrl1);
                let p3 = FloatPoint::from_point(to);
                match Segment::try_cubic_with_adapter(p0, p1, p2, p3, adapter)? {
                    TryCubicResult::None => {}
                    TryCubicResult::Segment(segment) => output.push(segment),
                    TryCubicResult::Segments(mut segments) => output.append(&mut segments),
                }
                Ok(to)
            }
        }
    }
}

impl<T: FloatNumber> Segment<T> {
    fn try_line_with_adapter<I: IntNumber>(
        p0: FloatPoint<T>,
        p1: FloatPoint<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Option<Self>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        if q0 != q1 {
            Ok(Some(Segment::Line(LineSegment {
                control_points: [p0, p1],
            })))
        } else {
            Ok(None)
        }
    }

    fn try_quad_with_adapter<I: IntNumber>(
        p0: FloatPoint<T>,
        p1: FloatPoint<T>,
        p2: FloatPoint<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Option<Self>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        if q0 == q1 && q1 == q2 {
            Ok(None)
        } else if q0 != q2 && Triangle::is_line(q0, q1, q2) {
            Self::try_line_with_adapter(p0, p2, adapter)
        } else {
            Ok(Some(Segment::Quad(QuadSegment {
                control_points: [p0, p1, p2],
            })))
        }
    }

    fn try_cubic_with_adapter<I: IntNumber>(
        p0: FloatPoint<T>,
        p1: FloatPoint<T>,
        p2: FloatPoint<T>,
        p3: FloatPoint<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<TryCubicResult<T>, FloatPointAdapterRangeError> {
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        let q3 = adapter.try_float_to_int(&p3)?;
        if q0 == q1 && q1 == q2 && q2 == q3 {
            return Ok(TryCubicResult::None);
        }

        if q0 == q3 {
            let cubic = CubicSegment {
                control_points: [p0, p1, p2, p3],
            };
            let [first, last] = cubic.split_at(T::HALF);
            let mut segments = Vec::with_capacity(2);
            Self::push_cubic_without_self_intersection(first, adapter, &mut segments)?;
            Self::push_cubic_without_self_intersection(last, adapter, &mut segments)?;
            return if segments.is_empty() {
                Ok(TryCubicResult::None)
            } else {
                Ok(TryCubicResult::Segments(segments))
            };
        }

        if q1 == q2 {
            return Ok(Self::try_quad_with_adapter(p0, p1, p3, adapter)?.into());
        }

        if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            return Ok(Self::try_line_with_adapter(p0, p3, adapter)?.into());
        }

        let segment = CubicSegment {
            control_points: [p0, p1, p2, p3],
        };

        match segment.resolve_self_intersection() {
            None => Ok(TryCubicResult::Segment(Segment::Cubic(segment))),
            Some(intersection) => {
                let segments = segment.split_self_intersecting_with_adapter(intersection, adapter)?;
                if segments.is_empty() {
                    Ok(TryCubicResult::None)
                } else {
                    Ok(TryCubicResult::Segments(segments))
                }
            }
        }
    }

    pub(super) fn push_split_cubic_part<I: IntNumber>(
        cubic: CubicSegment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        output: &mut Vec<Segment<T>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let [p0, _, _, p3] = cubic.control_points;
        if adapter.try_float_to_int(&p0)? == adapter.try_float_to_int(&p3)? {
            let [first, last] = cubic.split_at(T::HALF);
            Self::push_cubic_without_self_intersection(first, adapter, output)?;
            Self::push_cubic_without_self_intersection(last, adapter, output)?;
        } else {
            Self::push_cubic_without_self_intersection(cubic, adapter, output)?;
        }

        Ok(())
    }

    fn push_cubic_without_self_intersection<I: IntNumber>(
        cubic: CubicSegment<T>,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
        output: &mut Vec<Segment<T>>,
    ) -> Result<(), FloatPointAdapterRangeError> {
        let [p0, p1, p2, p3] = cubic.control_points;
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        let q3 = adapter.try_float_to_int(&p3)?;
        if q0 == q1 && q1 == q2 && q2 == q3 {
            return Ok(());
        }

        if q1 == q2 {
            if let Some(segment) = Self::try_quad_with_adapter(p0, p1, p3, adapter)? {
                output.push(segment);
            }
        } else if Triangle::is_line(q0, q1, q3) && Triangle::is_line(q0, q2, q3) {
            if let Some(segment) = Self::try_line_with_adapter(p0, p3, adapter)? {
                output.push(segment);
            }
        } else {
            output.push(Segment::Cubic(cubic));
        }

        Ok(())
    }
}

pub(crate) enum TryCubicResult<T: FloatNumber> {
    None,
    Segment(Segment<T>),
    Segments(Vec<Segment<T>>),
}

impl<T: FloatNumber> From<Option<Segment<T>>> for TryCubicResult<T> {
    fn from(segment: Option<Segment<T>>) -> Self {
        match segment {
            Some(segment) => Self::Segment(segment),
            None => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::curve::cubic::CubicSegment;
    use crate::kernel::curve::segment::Segment;
    use crate::normalization::cubic::CubicSelfIntersection;
    use i_overlay::i_float::adapter::FloatPointAdapter;
    use i_overlay::i_float::float::point::FloatPoint;

    #[test]
    fn split_self_intersecting_cubic_handles_endpoint_intersection() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(-3.0, -3.0);
        let p2 = FloatPoint::new(-3.0, -2.0);
        let p3 = FloatPoint::new(-2.0, -2.0);
        let points = [p0, p1, p2, p3];
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(points.iter());
        let intersection = CubicSelfIntersection {
            t0: 3.0 / 7.0,
            t1: 6.0 / 7.0,
            point: p0,
        };
        let cubic = CubicSegment {
            control_points: [p0, p1, p2, p3],
        };

        let segments = cubic
            .split_self_intersecting_with_adapter(intersection, &adapter)
            .expect("points must fit adapter rect");

        assert!(!segments.is_empty());
        for segment in segments {
            if let Segment::Cubic(segment) = segment {
                let q0 = adapter.float_to_int(&segment.control_points[0]);
                let q1 = adapter.float_to_int(&segment.control_points[3]);
                assert_ne!(q0, q1);
            }
        }
    }
}
