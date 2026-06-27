use crate::kernel::float::cross::overlap::find::CurveOverlap;
use crate::kernel::float::curve::quad::SubQuadSegment;
use alloc::vec::Vec;
use core::marker::PhantomData;
use i_overlay::i_float::float::number::FloatNumber;

#[derive(Debug, Clone, Copy)]
pub(crate) struct QuadQuadPair<T: FloatNumber> {
    pub(crate) quad0: SubQuadSegment<T>,
    pub(crate) quad1: SubQuadSegment<T>,
}

pub struct Solver<T: FloatNumber> {
    phantom_data: PhantomData<T>,
    grid_size: T,
    relative_epsilon: T,
    min_possible_size: T,
    pub(crate) quad_quad_stack: Vec<QuadQuadPair<T>>,
    pub(crate) quad_quad_pairs: Vec<QuadQuadPair<T>>,
}

pub enum IntersectionResult<T: FloatNumber> {
    None,
    Overlap(CurveOverlap<T>),
    Intersect,
}

impl<T: FloatNumber> Solver<T> {
    #[inline]
    pub(crate) fn with_grid_size(grid_size: T) -> Self {
        Self::with_grid_size_and_options(grid_size, Self::default_relative_epsilon(), grid_size)
    }

    #[inline]
    pub(crate) fn with_grid_size_and_options(
        grid_size: T,
        relative_epsilon: T,
        min_possible_size: T,
    ) -> Self {
        Self {
            phantom_data: PhantomData,
            grid_size,
            relative_epsilon,
            min_possible_size,
            quad_quad_stack: Vec::new(),
            quad_quad_pairs: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn grid_size(&self) -> T {
        self.grid_size
    }

    #[inline]
    pub(crate) fn relative_epsilon(&self) -> T {
        self.relative_epsilon
    }

    #[inline]
    pub(crate) fn min_possible_size(&self) -> T {
        self.min_possible_size
    }

    #[inline]
    fn default_relative_epsilon() -> T {
        T::from_float(1.0e-10)
    }
}
