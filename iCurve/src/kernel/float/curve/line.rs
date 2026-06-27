use crate::kernel::float::curve::param::FloatSegmentParam;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub struct FloatLineSegment<T: FloatNumber> {
    pub control_points: [FloatPoint<T>; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct SubLineSegment<T: FloatNumber> {
    pub line: FloatLineSegment<T>,
    pub t0: FloatSegmentParam<T>,
    pub t1: FloatSegmentParam<T>,
}

impl<T: FloatNumber> SubLineSegment<T> {
    pub fn with_line(line: FloatLineSegment<T>) -> Self {
        Self {
            line,
            t0: FloatSegmentParam::Start,
            t1: FloatSegmentParam::End,
        }
    }
}
