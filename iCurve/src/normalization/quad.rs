use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::triangle::Triangle;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::quad::QuadSegment;
use crate::kernel::curve::segment::Segment;

impl<T: FloatNumber> QuadSegment<T>{
    #[inline]
    pub(super) fn try_with_adapter<I: IntNumber>(self, adapter: &FloatPointAdapter<FloatPoint<T>, I>) -> Result<Option<Segment<T>>, FloatPointAdapterRangeError> {
        let [p0, p1, p2] = self.control_points;
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        let q2 = adapter.try_float_to_int(&p2)?;
        if q0 == q2 {
            Ok(None)
        } else if Triangle::is_line(q0, q1, q2) {
            LineSegment { control_points: [p0, p2] }.try_with_adapter(adapter)
        } else {
            Ok(Some(Segment::Quad(self)))
        }
    }
}