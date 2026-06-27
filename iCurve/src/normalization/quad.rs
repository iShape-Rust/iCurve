use crate::kernel::float::curve::line::FloatLineSegment;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use crate::kernel::float::curve::segment::FloatSegment;
use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;

impl<T: FloatNumber> FloatQuadSegment<T> {
    #[inline]
    pub(super) fn try_with_adapter<I: IntNumber>(
        self,
        adapter: &FloatPointAdapter<FloatPoint<T>, I>,
    ) -> Result<Option<FloatSegment<T>>, FloatPointAdapterRangeError> {
        let [p0, p1, p2] = self.control_points;
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;

        // Closed quadratic normalizes to an out-and-back spike.
        if q0 == q2 {
            Ok(None)
        } else if Triangle::is_line(q0, q1, q2) {
            // Collinear quadratic contributes the same edge as its chord.
            FloatLineSegment {
                control_points: [p0, p2],
            }
            .try_with_adapter(adapter)
        } else {
            Ok(Some(FloatSegment::Quad(self)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalization::test_utils::assert_control_points_eq;

    #[test]
    fn drops_closed_quad_spike() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(1.0, 0.0);
        let quad = FloatQuadSegment {
            control_points: [p0, p1, p0],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(quad.control_points.iter());

        let segment = quad.try_with_adapter(&adapter).unwrap();

        assert!(segment.is_none());
    }

    #[test]
    fn reduces_collinear_quad_to_line() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(1.0, 0.0);
        let p2 = FloatPoint::new(2.0, 0.0);
        let quad = FloatQuadSegment {
            control_points: [p0, p1, p2],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(quad.control_points.iter());

        let segment = quad.try_with_adapter(&adapter).unwrap();

        match segment {
            Some(FloatSegment::Line(segment)) => assert_control_points_eq(segment.control_points, [p0, p2]),
            _ => panic!("expected line segment"),
        }
    }

    #[test]
    fn keeps_non_degenerate_quad() {
        let p0 = FloatPoint::new(0.0, 0.0);
        let p1 = FloatPoint::new(1.0, 1.0);
        let p2 = FloatPoint::new(2.0, 0.0);
        let quad = FloatQuadSegment {
            control_points: [p0, p1, p2],
        };
        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::with_iter(quad.control_points.iter());

        let segment = quad.try_with_adapter(&adapter).unwrap();

        match segment {
            Some(FloatSegment::Quad(segment)) => {
                assert_control_points_eq(segment.control_points, [p0, p1, p2])
            }
            _ => panic!("expected quad segment"),
        }
    }
}
