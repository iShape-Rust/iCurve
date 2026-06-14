use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::rect::IntRect;

pub struct Collider<I: IntNumber> {
    index: usize, // index to in ranges and collider vecs
    rect: IntRect<I>,
}

// impl<I: IntNumber> Collider<I> {
//     pub fn with_range<P:FloatPointCompatible>(range_index: usize, vec) -> Self {
//         let convex = range.to_convex();
//         Collider {
//             range,
//             convex,
//         }
//     }
// }
