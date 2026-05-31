use crate::flatten::segment::SegmentRange;
use alloc::vec::Vec;
use i_overlay::i_float::float::number::FloatNumber;

pub(super) trait MergeSegmentRanges<F: FloatNumber> {
    fn merge(self, output: &mut Vec<Vec<SegmentRange<F>>>);
}

impl<F: FloatNumber> MergeSegmentRanges<F> for Vec<Vec<SegmentRange<F>>> {
    fn merge(mut self, output: &mut Vec<Vec<SegmentRange<F>>>) {
        debug_assert!(!self.is_empty());
        output.clear();
        if self.len() < 2 {
            output.push(self.remove(0));
            return;
        }

        let mut this = self[0].to_vec();

        output.push(self[0].to_vec());

        for next in self.into_iter().skip(1) {
            let mut i = this.len();
            while i > 0 {
                i -= 1;
                let lt = &mut this[i];
                if let Some(rt) = next.iter().find(|it| it.segment_index == lt.segment_index)
                    && lt.t1 == rt.t0
                {
                    lt.t1 = rt.t1;
                    this.swap_remove(i);
                }
            }
        }
    }
}
