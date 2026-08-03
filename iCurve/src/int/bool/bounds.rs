use crate::int::CurveInt;
use crate::int::bool::edge::CurveEdge;
use alloc::vec::Vec;
use i_key_sort::sort::one_key_cmp::OneKeyAndCmpSort;
use i_overlay::i_float::int::rect::IntRect;

#[derive(Debug, Clone, Copy)]
pub(super) struct CurveEdgeBounds<I: CurveInt> {
    pub(super) edge_index: usize,
    pub(super) rect: IntRect<I>,
}

pub(super) struct CurveBoundsBuffer<I: CurveInt> {
    pub(super) bounds: Vec<CurveEdgeBounds<I>>,
    bounds_buffer: Vec<CurveEdgeBounds<I>>,
    pub(super) active: Vec<CurveEdgeBounds<I>>,
}

impl<I: CurveInt + i_key_sort::sort::key::SortKey> CurveBoundsBuffer<I> {
    pub(super) fn new() -> Self {
        Self {
            bounds: Vec::new(),
            bounds_buffer: Vec::new(),
            active: Vec::new(),
        }
    }

    pub(super) fn build(&mut self, edges: &[CurveEdge<I>]) {
        self.bounds.clear();
        self.bounds
            .reserve(edges.len().saturating_sub(self.bounds.capacity()));

        for (edge_index, edge) in edges.iter().enumerate() {
            let hull = edge.curve.convex_hull();
            let rect = IntRect::with_points(hull.as_slice()).unwrap();
            self.bounds.push(CurveEdgeBounds { edge_index, rect });
        }

        self.bounds.sort_by_one_key_then_by_and_buffer(
            false,
            &mut self.bounds_buffer,
            |item| item.rect.min_x,
            |first, second| first.rect.min_y.cmp(&second.rect.min_y),
        );
    }
}
