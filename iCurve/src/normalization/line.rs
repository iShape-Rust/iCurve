use i_overlay::i_float::adapter::{FloatPointAdapter, FloatPointAdapterRangeError};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_float::int::number::int::IntNumber;
use crate::kernel::curve::line::LineSegment;
use crate::kernel::curve::segment::Segment;

impl<T: FloatNumber> LineSegment<T>{
    #[inline]
    pub(super) fn try_with_adapter<I: IntNumber>(self, adapter: &FloatPointAdapter<FloatPoint<T>, I>) -> Result<Option<Segment<T>>, FloatPointAdapterRangeError> {
        let [p0, p1] = self.control_points;
        let q0 = adapter.try_float_to_int(&p0)?;
        let q1 = adapter.try_float_to_int(&p1)?;
        if q0 != q1 {
            Ok(Some(Segment::Line(self)))
        } else {
            Ok(None)
        }
    }
}