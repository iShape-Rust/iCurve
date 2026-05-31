use crate::flatten::segment::{SegmentParam, SegmentRange};
use alloc::slice;
use alloc::vec::Vec;
use core::iter;
use i_overlay::core::edge_data::{EdgeDataMerge, EdgeDataSplit, OverlayEdgeData};
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::segm::boolean::ShapeCountBoolean;
use i_overlay::vector::edge::DataVectorShape;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedCurveOverlay<I: IntNumber, F: FloatNumber> {
    pub(crate) shapes: Vec<DataVectorShape<I, MetaSegment<F>>>,
    pub(crate) store: MetaStore<F>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MetaId(usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MetaSegment<F: FloatNumber> {
    Single(SegmentRange<F>),
    Multi(MetaId),
}

#[derive(Debug, Clone)]
pub(crate) struct MetaStore<F: FloatNumber> {
    sets: Vec<Vec<SegmentRange<F>>>,
}

impl<F: FloatNumber> MetaSegment<F> {
    #[inline(always)]
    pub(crate) fn single(segment: SegmentRange<F>) -> Self {
        Self::Single(segment)
    }

    pub(crate) fn with_segments(segments: Vec<SegmentRange<F>>, store: &mut MetaStore<F>) -> Self {
        debug_assert!(!segments.is_empty());
        if segments.len() == 1 {
            MetaSegment::Single(segments[0])
        } else {
            let id = MetaId(store.sets.len());
            store.sets.push(segments);
            MetaSegment::Multi(id)
        }
    }
}

impl<F: FloatNumber> Default for MetaStore<F> {
    fn default() -> Self {
        Self { sets: Vec::new() }
    }
}

impl<F: FloatNumber + Send + Sync> OverlayEdgeData for MetaSegment<F> {
    type Store = MetaStore<F>;

    #[inline(always)]
    fn reversed(mut self, store: &mut Self::Store) -> Self {
        match &mut self {
            Self::Single(segment) => segment.reverse(),
            Self::Multi(id) => {
                let vec = &mut store.sets[id.0];
                for s in vec.iter_mut() {
                    s.reverse()
                }
            }
        }

        self
    }

    #[inline(always)]
    fn split<I: IntNumber>(self, ctx: EdgeDataSplit<I>, store: &mut Self::Store) -> (Self, Self) {
        let ratio = split_ratio(ctx);
        let mut lhs = Vec::new();
        let mut rhs = Vec::new();

        for segment in store.range_iter(self) {
            let (s0, s1) = segment.split_at_ratio(ratio);
            lhs.push(s0);
            rhs.push(s1);
        }

        (Self::with_segments(lhs, store), Self::with_segments(rhs, store))
    }

    fn merge(ctx: EdgeDataMerge<ShapeCountBoolean, Self>, store: &mut Self::Store) -> Self {
        let mut segments: Vec<_> = store.range_iter(ctx.lhs_data).collect();
        let n = segments.len();
        for s in store.range_iter(ctx.rhs_data) {
            if !segments[0..n].contains(&s) {
                segments.push(s);
            }
        }

        Self::with_segments(segments, store)
    }
}

trait SegmentRangeMeta<F: FloatNumber> {
    fn split_at_ratio(self, ratio: f64) -> (Self, Self)
    where
        Self: Sized;
    fn reverse(&mut self);
}

impl<F: FloatNumber> SegmentRangeMeta<F> for SegmentRange<F> {
    fn split_at_ratio(self, ratio: f64) -> (Self, Self) {
        let tm = if ratio <= 0.0 {
            self.t0
        } else if ratio >= 1.0 {
            self.t1
        } else {
            let t0 = self.t0.value();
            let t1 = self.t1.value();
            SegmentParam::new(t0 + (t1 - t0) * F::from_float(ratio))
        };
        (
            Self {
                segment_index: self.segment_index,
                t0: self.t0,
                t1: tm,
            },
            Self {
                segment_index: self.segment_index,
                t0: tm,
                t1: self.t1,
            },
        )
    }

    fn reverse(&mut self) {
        let t0 = self.t0;
        self.t0 = self.t1;
        self.t1 = t0;
    }
}

fn split_ratio<I: IntNumber>(ctx: EdgeDataSplit<I>) -> f64 {
    let dx = ctx.b.x.wide() - ctx.a.x.wide();
    let dy = ctx.b.y.wide() - ctx.a.y.wide();

    let (num, den) = if dx.unsigned_abs() >= dy.unsigned_abs() {
        (ctx.p.x.wide() - ctx.a.x.wide(), dx)
    } else {
        (ctx.p.y.wide() - ctx.a.y.wide(), dy)
    };

    if den == I::Wide::ZERO {
        return 0.5;
    }

    (num.to_f64() / den.to_f64()).clamp(0.0, 1.0)
}

pub(crate) enum SegmentRangeIter<'a, F: FloatNumber> {
    Single(iter::Once<SegmentRange<F>>),
    Multi(slice::Iter<'a, SegmentRange<F>>),
}

impl<'a, F: FloatNumber> Iterator for SegmentRangeIter<'a, F>
where
    SegmentRange<F>: Clone,
{
    type Item = SegmentRange<F>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SegmentRangeIter::Single(iter) => iter.next(),
            SegmentRangeIter::Multi(iter) => iter.next().cloned(),
        }
    }
}

impl<F: FloatNumber> MetaStore<F> {
    #[inline(always)]
    pub(crate) fn range_iter(&self, meta: MetaSegment<F>) -> SegmentRangeIter<'_, F> {
        match meta {
            MetaSegment::Single(segment) => SegmentRangeIter::Single(iter::once(segment)),
            MetaSegment::Multi(id) => SegmentRangeIter::Multi(self.sets[id.0].iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn segment(segment_index: usize, t0: f64, t1: f64) -> SegmentRange<f64> {
        SegmentRange::new(segment_index, t0, t1)
    }

    #[test]
    fn meta_store_compacts_single_segment() {
        let mut store = MetaStore::default();

        let meta = MetaSegment::with_segments(vec![segment(1, 0.0, 1.0)], &mut store);

        assert_eq!(meta, MetaSegment::Single(segment(1, 0.0, 1.0)));
    }

    #[test]
    fn meta_segment_reverse_reverses_all_ranges() {
        let mut store = MetaStore::default();
        let meta = MetaSegment::with_segments(vec![segment(1, 0.0, 0.4), segment(2, 0.2, 0.8)], &mut store);

        let reversed = meta.reversed(&mut store);

        assert_eq!(
            store.range_iter(reversed).collect::<Vec<_>>(),
            vec![segment(1, 0.4, 0.0), segment(2, 0.8, 0.2)]
        );
    }
}
