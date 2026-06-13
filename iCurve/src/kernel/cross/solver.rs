use core::marker::PhantomData;
use i_overlay::i_float::float::number::FloatNumber;

pub struct Solver<T: FloatNumber> {
    phantom_data: PhantomData<T>,
    grid_size: T,
}

impl<T: FloatNumber> Solver<T> {
    #[inline]
    pub(crate) fn grid_size(&self) -> T {
        self.grid_size
    }
}
