use core::marker::PhantomData;
use i_overlay::i_float::float::number::FloatNumber;

pub struct Solver<T: FloatNumber> {
    phantom_data: PhantomData<T>
}
