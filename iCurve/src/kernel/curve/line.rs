use crate::kernel::curve::param::SegmentParam;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub struct LineSegment<T: FloatNumber> {
    pub control_points: [FloatPoint<T>; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct SubLineSegment<T: FloatNumber> {
    pub line: LineSegment<T>,
    pub t0: SegmentParam<T>,
    pub t1: SegmentParam<T>,
}

impl<T: FloatNumber> SubLineSegment<T> {
    pub fn with_line(line: LineSegment<T>) -> Self {
        Self {
            line,
            t0: SegmentParam::Start,
            t1: SegmentParam::End,
        }
    }
}
