use crate::flatten::segment::SubSegment;
use alloc::vec::Vec;
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
    Single(SubSegment<F>),
    Multi(MetaId),
}

#[derive(Debug, Clone)]
pub(crate) struct MetaStore<F: FloatNumber> {
    sets: Vec<Vec<SubSegment<F>>>,
}

impl<F: FloatNumber> MetaSegment<F> {
    #[inline(always)]
    pub(crate) fn single(segment: SubSegment<F>) -> Self {
        Self::Single(segment)
    }
}

impl<F: FloatNumber> MetaStore<F> {
    #[inline(always)]
    pub(crate) fn to_vec(&self, meta: MetaSegment<F>) -> Vec<SubSegment<F>> {
        match meta {
            MetaSegment::Single(segment) => Vec::from([segment]),
            MetaSegment::Multi(id) => self.sets[id.0].clone(),
        }
    }

    fn from_segments(&mut self, mut segments: Vec<SubSegment<F>>) -> MetaSegment<F> {
        dedup_segments(&mut segments);

        if segments.len() == 1 {
            return MetaSegment::Single(segments[0]);
        }

        let id = MetaId(self.sets.len());
        self.sets.push(segments);
        MetaSegment::Multi(id)
    }
}

impl<F: FloatNumber> Default for MetaStore<F> {
    fn default() -> Self {
        Self { sets: Vec::new() }
    }
}

impl<F: FloatNumber + Send + Sync> OverlayEdgeData for MetaSegment<F> {
    type Store = MetaStore<F>;

    fn merge(ctx: EdgeDataMerge<ShapeCountBoolean, Self>, store: &mut Self::Store) -> Self {
        let mut segments = store.to_vec(ctx.lhs_data);
        segments.extend(store.to_vec(ctx.rhs_data));
        store.from_segments(segments)
    }

    #[inline(always)]
    fn reversed(self, store: &mut Self::Store) -> Self {
        match self {
            Self::Single(segment) => Self::Single(segment.reversed()),
            Self::Multi(_) => {
                let segments = store
                    .to_vec(self)
                    .into_iter()
                    .map(SubSegmentMeta::reversed)
                    .collect();
                store.from_segments(segments)
            }
        }
    }

    #[inline(always)]
    fn split<I: IntNumber>(self, ctx: EdgeDataSplit<I>, store: &mut Self::Store) -> (Self, Self) {
        let ratio = split_ratio(ctx);
        let mut lhs = Vec::new();
        let mut rhs = Vec::new();

        for segment in store.to_vec(self) {
            let (s0, s1) = segment.split_at_ratio(ratio);
            lhs.push(s0);
            rhs.push(s1);
        }

        (store.from_segments(lhs), store.from_segments(rhs))
    }
}

trait SubSegmentMeta<F: FloatNumber> {
    fn reversed(self) -> Self;
    fn split_at_ratio(self, ratio: f64) -> (Self, Self)
    where
        Self: Sized;
}

impl<F: FloatNumber> SubSegmentMeta<F> for SubSegment<F> {
    fn reversed(self) -> Self {
        Self {
            segment_index: self.segment_index,
            t0: self.t1,
            t1: self.t0,
        }
    }

    fn split_at_ratio(self, ratio: f64) -> (Self, Self) {
        let tm = self.t0 + (self.t1 - self.t0) * F::from_float(ratio);
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
}

fn dedup_segments<F: FloatNumber>(segments: &mut Vec<SubSegment<F>>) {
    let mut i = 0;
    while i < segments.len() {
        let segment = segments[i];
        if segments[..i].contains(&segment) {
            segments.remove(i);
        } else {
            i += 1;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(segment_index: usize, t0: f64, t1: f64) -> SubSegment<f64> {
        SubSegment {
            segment_index,
            t0,
            t1,
        }
    }

    #[test]
    fn meta_store_compacts_single_segment() {
        let mut store = MetaStore::default();
        let meta = store.from_segments(Vec::from([segment(1, 0.0, 1.0)]));

        assert_eq!(meta, MetaSegment::Single(segment(1, 0.0, 1.0)));
    }

    #[test]
    fn meta_store_deduplicates_multi_segments() {
        let mut store = MetaStore::default();
        let meta = store.from_segments(Vec::from([
            segment(1, 0.0, 0.5),
            segment(2, 0.0, 0.5),
            segment(1, 0.0, 0.5),
        ]));

        assert_eq!(
            store.to_vec(meta),
            Vec::from([segment(1, 0.0, 0.5), segment(2, 0.0, 0.5)])
        );
    }

    #[test]
    fn meta_segment_reverse_reverses_all_ranges() {
        let mut store = MetaStore::default();
        let meta = store.from_segments(Vec::from([segment(1, 0.0, 0.4), segment(2, 0.2, 0.8)]));

        let reversed = meta.reversed(&mut store);

        assert_eq!(
            store.to_vec(reversed),
            Vec::from([segment(1, 0.4, 0.0), segment(2, 0.8, 0.2)])
        );
    }
}
