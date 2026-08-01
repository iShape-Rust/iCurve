use crate::int::CurveInt;
use crate::int::bool::edge::CurveEdge;
use crate::int::bool::source::CurveId;
use crate::kernel::int::curve::chord::SegmentChord;
use crate::kernel::int::curve::param::SegmentParam;
use alloc::boxed::Box;
use alloc::vec::Vec;
use i_overlay::core::edge_data::{EdgeDataMerge, EdgeDataSplit, OverlayEdgeData};
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;

/// Source parameters use one common scale so edge data stays independent of
/// the coordinate type selected by `iOverlay::OverlayEdgeData::split`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CurveParam(u64);

impl CurveParam {
    const SCALE: u32 = 62;
    const DENOMINATOR: u64 = 1_u64 << Self::SCALE;

    #[inline]
    pub(crate) fn from_segment<I: IntNumber>(param: SegmentParam<I>) -> Self {
        let value = wide_to_u64::<I>(param.value());
        Self(value << (Self::SCALE - SegmentParam::<I>::SCALE))
    }

    #[inline]
    pub(crate) fn to_segment<I: IntNumber>(self) -> SegmentParam<I> {
        let shift = Self::SCALE - SegmentParam::<I>::SCALE;
        let value = if shift == 0 {
            self.0
        } else {
            (self.0 + (1_u64 << (shift - 1))) >> shift
        };
        let wide = u64_to_wide::<I>(value);
        SegmentParam::new(I::from_wide(wide))
    }

    #[inline]
    fn interpolate(start: Self, end: Self, local: Self) -> Self {
        let span = end.0 as i128 - start.0 as i128;
        let product = span * local.0 as i128;
        let denominator = Self::DENOMINATOR as i128;
        let half = denominator >> 1;
        let offset = if product >= 0 {
            (product + half) / denominator
        } else {
            -((-product + half) / denominator)
        };
        let value = (start.0 as i128 + offset).clamp(0, Self::DENOMINATOR as i128);
        Self(value as u64)
    }
}

#[inline]
fn wide_to_u64<I: IntNumber>(value: I::Wide) -> u64 {
    const CHUNK_BITS: u32 = 31;
    const CHUNK_MASK: u32 = (1_u32 << CHUNK_BITS) - 1;

    debug_assert!(value >= I::Wide::ZERO);
    let mask = I::Wide::from_u32(CHUNK_MASK);
    let low = (value & mask).to_usize() as u64;
    let high = (value >> CHUNK_BITS).to_usize() as u64;
    (high << CHUNK_BITS) | low
}

#[inline]
fn u64_to_wide<I: IntNumber>(value: u64) -> I::Wide {
    const CHUNK_BITS: u32 = 31;
    const CHUNK_MASK: u64 = (1_u64 << CHUNK_BITS) - 1;

    let low = I::Wide::from_u32((value & CHUNK_MASK) as u32);
    let high = I::Wide::from_u32((value >> CHUNK_BITS) as u32);
    (high << CHUNK_BITS) + low
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CurveSourceSpan {
    pub(crate) curve_id: CurveId,
    pub(crate) start: CurveParam,
    pub(crate) end: CurveParam,
}

impl CurveSourceSpan {
    #[inline]
    pub(crate) fn from_edge<I: CurveInt>(edge: CurveEdge<I>) -> Self {
        Self {
            curve_id: edge.curve_id,
            start: CurveParam::from_segment(edge.start_param),
            end: CurveParam::from_segment(edge.end_param),
        }
    }

    #[inline]
    pub(crate) fn reversed(self) -> Self {
        Self {
            curve_id: self.curve_id,
            start: self.end,
            end: self.start,
        }
    }

    #[inline]
    pub(crate) fn is_collapsed(self) -> bool {
        self.start == self.end
    }

    #[inline]
    fn split(self, local: CurveParam) -> (Self, Self) {
        let middle = CurveParam::interpolate(self.start, self.end, local);
        (
            Self {
                curve_id: self.curve_id,
                start: self.start,
                end: middle,
            },
            Self {
                curve_id: self.curve_id,
                start: middle,
                end: self.end,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CurveEdgeData {
    Single(CurveSourceSpan),
    Multi(CurveSetId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurveSetId(usize);

#[derive(Debug, Default)]
pub(crate) struct CurveEdgeDataStore {
    sets: Vec<Box<[CurveSourceSpan]>>,
}

impl CurveEdgeDataStore {
    pub(crate) fn spans(&self, data: CurveEdgeData, buffer: &mut Vec<CurveSourceSpan>) {
        buffer.clear();
        self.append(data, buffer);
    }

    fn append(&self, data: CurveEdgeData, spans: &mut Vec<CurveSourceSpan>) {
        match data {
            CurveEdgeData::Single(span) => spans.push(span),
            CurveEdgeData::Multi(set_id) => spans.extend_from_slice(&self.sets[set_id.0]),
        }
    }

    fn intern(&mut self, mut spans: Vec<CurveSourceSpan>) -> CurveEdgeData {
        spans.sort_unstable();
        spans.dedup();

        if spans.len() == 1 {
            return CurveEdgeData::Single(spans[0]);
        }

        if let Some(index) = self.sets.iter().position(|set| set.as_ref() == spans) {
            return CurveEdgeData::Multi(CurveSetId(index));
        }

        let set_id = CurveSetId(self.sets.len());
        self.sets.push(spans.into_boxed_slice());
        CurveEdgeData::Multi(set_id)
    }

    fn merge(&mut self, lhs: CurveEdgeData, rhs: CurveEdgeData) -> CurveEdgeData {
        if lhs == rhs {
            return lhs;
        }

        let mut spans = Vec::new();
        self.append(lhs, &mut spans);
        self.append(rhs, &mut spans);
        self.intern(spans)
    }

    fn reversed(&mut self, data: CurveEdgeData) -> CurveEdgeData {
        match data {
            CurveEdgeData::Single(span) => CurveEdgeData::Single(span.reversed()),
            CurveEdgeData::Multi(_) => {
                let mut spans = Vec::new();
                self.append(data, &mut spans);
                for span in &mut spans {
                    *span = span.reversed();
                }
                self.intern(spans)
            }
        }
    }

    fn split<I: IntNumber>(
        &mut self,
        data: CurveEdgeData,
        ctx: EdgeDataSplit<I>,
    ) -> (CurveEdgeData, CurveEdgeData) {
        let chord = SegmentChord { a: ctx.a, b: ctx.b };
        let local = CurveParam::from_segment(chord.param_for_point(ctx.p));

        match data {
            CurveEdgeData::Single(span) => {
                let (left, right) = span.split(local);
                (CurveEdgeData::Single(left), CurveEdgeData::Single(right))
            }
            CurveEdgeData::Multi(_) => {
                let mut spans = Vec::new();
                self.append(data, &mut spans);
                let mut left = Vec::with_capacity(spans.len());
                let mut right = Vec::with_capacity(spans.len());
                for span in spans {
                    let (left_span, right_span) = span.split(local);
                    left.push(left_span);
                    right.push(right_span);
                }
                (self.intern(left), self.intern(right))
            }
        }
    }
}

impl<C> OverlayEdgeData<C> for CurveEdgeData {
    type Store = CurveEdgeDataStore;

    #[inline]
    fn reversed(self, store: &mut Self::Store) -> Self {
        store.reversed(self)
    }

    #[inline]
    fn split<I: IntNumber>(self, ctx: EdgeDataSplit<I>, store: &mut Self::Store) -> (Self, Self) {
        store.split(self, ctx)
    }

    #[inline]
    fn merge(ctx: EdgeDataMerge<C, Self>, store: &mut Self::Store) -> Self {
        store.merge(ctx.lhs_data, ctx.rhs_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use i_overlay::i_shape::int::IntPoint;
    use i_overlay::segm::boolean::ShapeCountBoolean;

    fn param(value: i32, denominator: i32) -> CurveParam {
        CurveParam::from_segment(SegmentParam::<i32>::from_int(value, denominator))
    }

    fn span(id: usize, start: (i32, i32), end: (i32, i32)) -> CurveSourceSpan {
        CurveSourceSpan {
            curve_id: CurveId(id),
            start: param(start.0, start.1),
            end: param(end.0, end.1),
        }
    }

    fn merge(lhs: CurveEdgeData, rhs: CurveEdgeData, store: &mut CurveEdgeDataStore) -> CurveEdgeData {
        let empty = ShapeCountBoolean { subj: 0, clip: 0 };
        CurveEdgeData::merge(
            EdgeDataMerge {
                lhs_data: lhs,
                lhs_count: empty,
                rhs_data: rhs,
                rhs_count: empty,
                out_count: empty,
            },
            store,
        )
    }

    #[test]
    fn segment_param_round_trips_for_each_integer_width() {
        let i16_param = SegmentParam::<i16>::from_int(1, 3);
        let i32_param = SegmentParam::<i32>::from_int(1, 3);
        let i64_param = SegmentParam::<i64>::from_int(1, 3);

        assert_eq!(CurveParam::from_segment(i16_param).to_segment::<i16>(), i16_param);
        assert_eq!(CurveParam::from_segment(i32_param).to_segment::<i32>(), i32_param);
        assert_eq!(CurveParam::from_segment(i64_param).to_segment::<i64>(), i64_param);
    }

    #[test]
    fn split_interpolates_forward_source_span() {
        let mut store = CurveEdgeDataStore::default();
        let data = CurveEdgeData::Single(span(4, (1, 4), (3, 4)));

        let (left, right) = <CurveEdgeData as OverlayEdgeData<ShapeCountBoolean>>::split(
            data,
            EdgeDataSplit {
                a: IntPoint::new(0_i32, 0),
                p: IntPoint::new(5, 0),
                b: IntPoint::new(10, 0),
            },
            &mut store,
        );

        let CurveEdgeData::Single(left) = left else {
            panic!("expected one left span");
        };
        let CurveEdgeData::Single(right) = right else {
            panic!("expected one right span");
        };
        assert_eq!(left.start, param(1, 4));
        assert_eq!(left.end, param(1, 2));
        assert_eq!(right.start, left.end);
        assert_eq!(right.end, param(3, 4));
    }

    #[test]
    fn split_interpolates_reversed_source_span() {
        let mut store = CurveEdgeDataStore::default();
        let data = CurveEdgeData::Single(span(4, (3, 4), (1, 4)));

        let (left, right) = <CurveEdgeData as OverlayEdgeData<ShapeCountBoolean>>::split(
            data,
            EdgeDataSplit {
                a: IntPoint::new(0_i32, 0),
                p: IntPoint::new(5, 0),
                b: IntPoint::new(10, 0),
            },
            &mut store,
        );

        let (CurveEdgeData::Single(left), CurveEdgeData::Single(right)) = (left, right) else {
            panic!("expected single spans");
        };
        assert_eq!(left.start, param(3, 4));
        assert_eq!(left.end, param(1, 2));
        assert_eq!(right.start, left.end);
        assert_eq!(right.end, param(1, 4));
    }

    #[test]
    fn split_preserves_a_collapsed_child_range() {
        let mut store = CurveEdgeDataStore::default();
        let start = CurveParam(1_000);
        let end = CurveParam(1_001);
        let data = CurveEdgeData::Single(CurveSourceSpan {
            curve_id: CurveId(2),
            start,
            end,
        });

        let (left, right) = <CurveEdgeData as OverlayEdgeData<ShapeCountBoolean>>::split(
            data,
            EdgeDataSplit {
                a: IntPoint::new(0_i32, 0),
                p: IntPoint::new(1, 0),
                b: IntPoint::new(2, 0),
            },
            &mut store,
        );

        let (CurveEdgeData::Single(left), CurveEdgeData::Single(right)) = (left, right) else {
            panic!("expected single spans");
        };
        assert_eq!(left.start, start);
        assert_eq!(left.end, end);
        assert_eq!(right.start, end);
        assert_eq!(right.end, end);
        assert!(right.is_collapsed());
    }

    #[test]
    fn split_updates_every_merged_source_span() {
        let mut store = CurveEdgeDataStore::default();
        let first = CurveEdgeData::Single(span(1, (0, 1), (1, 1)));
        let second = CurveEdgeData::Single(span(2, (1, 1), (0, 1)));
        let data = merge(first, second, &mut store);

        let (left, right) = <CurveEdgeData as OverlayEdgeData<ShapeCountBoolean>>::split(
            data,
            EdgeDataSplit {
                a: IntPoint::new(0_i32, 0),
                p: IntPoint::new(1, 0),
                b: IntPoint::new(4, 0),
            },
            &mut store,
        );
        let mut left_spans = Vec::new();
        let mut right_spans = Vec::new();
        store.spans(left, &mut left_spans);
        store.spans(right, &mut right_spans);

        assert_eq!(left_spans.len(), 2);
        assert_eq!(right_spans.len(), 2);
        for left_span in left_spans {
            let right_span = right_spans
                .iter()
                .find(|span| span.curve_id == left_span.curve_id)
                .unwrap();
            assert_eq!(left_span.end, right_span.start);
        }
    }

    #[test]
    fn merge_preserves_distinct_ranges_of_the_same_curve() {
        let mut store = CurveEdgeDataStore::default();
        let first = CurveEdgeData::Single(span(1, (0, 1), (1, 4)));
        let second = CurveEdgeData::Single(span(1, (3, 4), (1, 1)));

        let data = merge(first, second, &mut store);
        let mut spans = Vec::new();
        store.spans(data, &mut spans);

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].curve_id, CurveId(1));
        assert_eq!(spans[1].curve_id, CurveId(1));
        assert_ne!(spans[0], spans[1]);
    }

    #[test]
    fn reverse_swaps_every_source_range() {
        let mut store = CurveEdgeDataStore::default();
        let first = CurveEdgeData::Single(span(1, (0, 1), (1, 4)));
        let second = CurveEdgeData::Single(span(2, (1, 2), (3, 4)));
        let data = merge(first, second, &mut store);

        let reversed = <CurveEdgeData as OverlayEdgeData<ShapeCountBoolean>>::reversed(data, &mut store);
        let mut spans = Vec::new();
        store.spans(reversed, &mut spans);

        assert!(spans.contains(&span(1, (1, 4), (0, 1))));
        assert!(spans.contains(&span(2, (3, 4), (1, 2))));
    }
}
