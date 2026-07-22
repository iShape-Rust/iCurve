use crate::kernel::int::curve::chord::Chord;
use crate::kernel::int::curve::param::SegmentParam;
use crate::kernel::int::curve::segment::Segment;
use alloc::vec;
use alloc::vec::Vec;
use i_overlay::core::overlay::ShapeType;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_shape::int::IntPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CurveId(pub(crate) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurveMark<I: IntNumber> {
    pub(crate) point: IntPoint<I>,
    pub(crate) param: SegmentParam<I>,
}

#[derive(Debug, Clone)]
pub(crate) struct CurveSlice<I: IntNumber> {
    pub(crate) curve: Segment<I>,
    pub(crate) shape_type: ShapeType,
    pub(crate) marks: Vec<CurveMark<I>>,
}

impl<I: IntNumber> CurveSlice<I> {
    pub(crate) fn new(curve: Segment<I>, shape_type: ShapeType) -> Self {
        let chord = curve.chord();
        Self {
            curve,
            shape_type,
            marks: vec![
                CurveMark {
                    point: chord.a,
                    param: SegmentParam::new(I::ZERO),
                },
                CurveMark {
                    point: chord.b,
                    param: SegmentParam::new(I::from_wide(SegmentParam::<I>::DENOMINATOR)),
                },
            ],
        }
    }

    #[inline]
    pub(crate) fn param_at(&self, point: IntPoint<I>) -> Option<SegmentParam<I>> {
        self.marks
            .iter()
            .find(|mark| mark.point == point)
            .map(|mark| mark.param)
    }

    #[inline]
    pub(crate) fn add_mark(&mut self, mark: CurveMark<I>) {
        if self.param_at(mark.point).is_none() {
            self.marks.push(mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::int::curve::line::LineSegment;

    #[test]
    fn keeps_first_param_for_a_point_without_sorting() {
        let point = IntPoint::new(5, 0);
        let mut slice = CurveSlice::new(
            Segment::Line(LineSegment {
                control_points: [IntPoint::new(0, 0), IntPoint::new(10, 0)],
            }),
            ShapeType::Subject,
        );

        slice.add_mark(CurveMark {
            point,
            param: SegmentParam::from_int(1, 2),
        });
        slice.add_mark(CurveMark {
            point,
            param: SegmentParam::from_int(2, 3),
        });

        assert_eq!(slice.marks.len(), 3);
        assert_eq!(slice.param_at(point), Some(SegmentParam::from_int(1, 2)));
    }
}
