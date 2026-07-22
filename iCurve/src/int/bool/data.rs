use crate::int::bool::slice::CurveId;
use alloc::boxed::Box;
use alloc::vec::Vec;
use i_overlay::core::edge_data::{EdgeDataMerge, OverlayEdgeData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CurveEdgeData {
    Single(CurveId),
    Multi(CurveSetId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CurveSetId(usize);

#[derive(Debug, Default)]
pub(crate) struct CurveEdgeDataStore {
    sets: Vec<Box<[CurveId]>>,
}

impl CurveEdgeDataStore {
    pub(crate) fn curve_ids(&self, data: CurveEdgeData, buffer: &mut Vec<CurveId>) {
        buffer.clear();
        self.append(data, buffer);
    }

    fn append(&self, data: CurveEdgeData, ids: &mut Vec<CurveId>) {
        match data {
            CurveEdgeData::Single(curve_id) => ids.push(curve_id),
            CurveEdgeData::Multi(set_id) => ids.extend_from_slice(&self.sets[set_id.0]),
        }
    }

    fn intern(&mut self, mut ids: Vec<CurveId>) -> CurveEdgeData {
        ids.sort_unstable_by_key(|id| id.0);
        ids.dedup();

        if ids.len() == 1 {
            return CurveEdgeData::Single(ids[0]);
        }

        if let Some(index) = self.sets.iter().position(|set| set.as_ref() == ids) {
            return CurveEdgeData::Multi(CurveSetId(index));
        }

        let set_id = CurveSetId(self.sets.len());
        self.sets.push(ids.into_boxed_slice());
        CurveEdgeData::Multi(set_id)
    }

    fn merge(&mut self, lhs: CurveEdgeData, rhs: CurveEdgeData) -> CurveEdgeData {
        if lhs == rhs {
            return lhs;
        }

        let mut ids = Vec::new();
        self.append(lhs, &mut ids);
        self.append(rhs, &mut ids);
        self.intern(ids)
    }
}

impl<C> OverlayEdgeData<C> for CurveEdgeData {
    type Store = CurveEdgeDataStore;

    #[inline]
    fn merge(ctx: EdgeDataMerge<C, Self>, store: &mut Self::Store) -> Self {
        store.merge(ctx.lhs_data, ctx.rhs_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use i_overlay::segm::boolean::ShapeCountBoolean;

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
    fn merge_builds_sorted_unique_set() {
        let mut store = CurveEdgeDataStore::default();
        let a = CurveEdgeData::Single(CurveId(4));
        let b = CurveEdgeData::Single(CurveId(1));

        let data = merge(a, b, &mut store);

        assert_eq!(data, CurveEdgeData::Multi(CurveSetId(0)));
        assert_eq!(store.sets[0].as_ref(), &[CurveId(1), CurveId(4)]);
        assert_eq!(merge(data, a, &mut store), data);
    }

    #[test]
    fn equal_sets_are_interned() {
        let mut store = CurveEdgeDataStore::default();
        let a = CurveEdgeData::Single(CurveId(0));
        let b = CurveEdgeData::Single(CurveId(1));
        let c = CurveEdgeData::Single(CurveId(2));

        let ab = merge(a, b, &mut store);
        let abc_from_ab = merge(ab, c, &mut store);
        let bc = merge(b, c, &mut store);
        let abc_from_bc = merge(a, bc, &mut store);

        assert_eq!(abc_from_ab, abc_from_bc);
        assert_eq!(store.sets.len(), 3);
    }

    #[test]
    fn reverse_preserves_candidates() {
        let mut store = CurveEdgeDataStore::default();
        let data = CurveEdgeData::Single(CurveId(7));

        let reversed = <CurveEdgeData as OverlayEdgeData<ShapeCountBoolean>>::reversed(data, &mut store);
        assert_eq!(reversed, data);
    }
}
