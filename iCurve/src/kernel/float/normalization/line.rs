use crate::kernel::float::curve::line::FloatLineSegment;
use crate::kernel::float::curve::segment::FloatSegment;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;

impl<T: FloatNumber> FloatLineSegment<T> {
    #[inline]
    pub(crate) fn try_with_adapter<I: IntNumber>(
        self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Option<FloatSegment<T>>, FloatPointAdapterRangeError> {
        let [p0, p1] = self.control_points;
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;

        // Same normalized endpoint: zero-length edge.
        if q0 != q1 {
            Ok(Some(FloatSegment::Line(self)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::float::normalization::test_utils::assert_control_points_eq;

    #[test]
    fn drops_zero_length_line() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let line = FloatLineSegment {
            control_points: [p0, p0],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(line.control_points.iter());

        let segment = line.try_with_adapter(&adapter).unwrap();

        assert!(segment.is_none());
    }

    #[test]
    fn keeps_non_zero_line() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(1.0, 0.0);
        let line = FloatLineSegment {
            control_points: [p0, p1],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(line.control_points.iter());

        let segment = line.try_with_adapter(&adapter).unwrap();

        match segment {
            Some(FloatSegment::Line(segment)) => assert_control_points_eq(segment.control_points, [p0, p1]),
            _ => panic!("expected line segment"),
        }
    }
}
