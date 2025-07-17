use alloc::vec;
use alloc::vec::Vec;
use crate::data::link_list::{LinkList, EMPTY_REF};
use crate::int::bezier::spline::IntCADSpline;
use crate::int::math::normalize::{Normalize16, UNIT, VALUABLE_BITS};
use crate::int::math::point::IntPoint;

#[derive(Copy, Clone)]
pub struct IntShort {
    pub step: usize,
    pub split_factor: u32,
    pub dir: IntPoint,
    pub a: IntPoint,
    pub b: IntPoint,
}

//https://pomax.github.io/bezierinfo/
pub trait IntSplineShorts {
    fn approximate(&self, min_cos: u32, min_len: u32) -> Vec<IntShort>;
}

impl<Spline: IntCADSpline> IntSplineShorts for Spline {
    #[inline]
    fn approximate(&self, min_cos: u32, min_len: u32) -> Vec<IntShort> {
        debug_assert!(min_cos <= UNIT);

        let mut segments = LinkList::new(vec![IntShort {
            step: 0,
            split_factor: 0,
            dir: (self.end() - self.start()).normalized_16bit(),
            a: self.start(),
            b: self.end(),
        }]);

        let shifted_min_cos = (min_cos as i64) << VALUABLE_BITS;
        segments.approximate(self, shifted_min_cos, min_len);

        let mut shorts = Vec::with_capacity(segments.len());
        let mut index = 0;
        while index != EMPTY_REF {
            let node = segments.get(index);
            shorts.push(node.item);
            index = node.next;
        }
        
        shorts
    }
}

impl LinkList<IntShort> {

    #[inline]
    fn approximate<Spline: IntCADSpline>(&mut self, spline: &Spline, shifted_min_cos: i64, min_len: u32) -> Vec<IntShort> {
        let min_len_power = min_len.ilog2();
        let st_dir = spline.start_dir();
        let ed_dir = spline.end_dir();

        let mut buffer = Vec::with_capacity(16);
        buffer.push(0);

        let mut to_split = Vec::with_capacity(16);

        while !buffer.is_empty() {
            for &index in buffer.iter() {
                if self.split_test(index, &st_dir, &ed_dir, shifted_min_cos) {
                    to_split.push(index);
                }
            }

            buffer.clear();
            for &index in to_split.iter() {
                self.split(spline, index, min_len_power, &mut buffer);
            }
            to_split.clear();
        }

        let mut shorts = Vec::with_capacity(self.len());
        let mut index = 0;
        while index != EMPTY_REF {
            let node = self.get(index);
            shorts.push(node.item);
            index = node.next
        }

        shorts
    }

    fn split_test(&self, index: u32, st_dir: &IntPoint, ed_dir: &IntPoint, shifted_min_cos: i64) -> bool {
        let node = self.get(index);
        let prev = node.prev;
        let next = node.next;
        let dir = node.item.dir;
        let prev_dir = if prev != EMPTY_REF {
            self.get(prev).item.dir
        } else {
            st_dir.clone()
        };

        let prev_dot_product = dir.dot_product(&prev_dir);
        if prev_dot_product < shifted_min_cos {
            return true;
        }

        let next_dir = if next != EMPTY_REF {
            self.get(next).item.dir
        } else {
            ed_dir.clone()
        };

        let next_dot_product = dir.dot_product(&next_dir);
        next_dot_product < shifted_min_cos
    }

    fn split<Spline: IntCADSpline>(&mut self, spline: &Spline, index: u32, min_len_power: u32, result: &mut Vec<u32>) {
        let short = self.get(index).item;

        let split_factor = short.split_factor + 1;
        let m = spline.split_at(short.step + 1, split_factor);
        let ma = m - short.a;
        let bm = short.b - m;

        if short.a == m || short.b == m {
            // split is not possible
            return;
        }

        let s0 = IntShort {
            step: short.step << 1,
            split_factor,
            dir: ma.normalized_16bit(),
            a: short.a,
            b: m,
        };

        let s1 = IntShort {
            step: (short.step + 1) << 1,
            split_factor,
            dir: bm.normalized_16bit(),
            a: m,
            b: short.b,
        };

        let (i0, i1) = self.split_at(index, s0, s1);

        if !ma.is_small(min_len_power) {
            result.push(i0)
        }

        if !bm.is_small(min_len_power) {
            result.push(i1)
        }
    }

}

impl IntPoint {
    #[inline]
    fn is_small(&self, power: u32) -> bool {
        debug_assert!(power < 30);
        let xx = self.x.unsigned_abs().max(1).ilog2();
        let yy = self.y.unsigned_abs().max(1).ilog2();
        let is_x_small = xx < power;
        let is_y_small = yy < power;
        if is_x_small && is_y_small {
            // we safe to use i64
            let len = (self.x.pow(2) + self.y.pow(2)).isqrt();
            len.ilog2() < power
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::int::bezier::short::IntSplineShorts;
    use crate::int::bezier::spline_cube::IntCubeSpline;
    use crate::int::math::normalize::normalize_unit_value;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_01() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 50),
            bm: IntPoint::new(100, 50),
            b: IntPoint::new(100, 0),
        };

        let shorts = spline.approximate(normalize_unit_value(0.8), 8);
        assert_eq!(shorts.len(), 8);
    }

    #[test]
    fn test_02() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 50),
            bm: IntPoint::new(100, 50),
            b: IntPoint::new(100, 0),
        };

        let shorts = spline.approximate(normalize_unit_value(0.8), 32);
        assert_eq!(shorts.len(), 6);
    }

    #[test]
    fn test_03() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 50),
            bm: IntPoint::new(100, 50),
            b: IntPoint::new(100, 0),
        };

        let shorts = spline.approximate(normalize_unit_value(0.9), 4);
        assert_eq!(shorts.len(), 8);
    }

    #[test]
    fn test_04() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 512),
            bm: IntPoint::new(512, 1024),
            b: IntPoint::new(1024, 1024),
        };

        let shorts = spline.approximate(normalize_unit_value(0.9), 16);
        assert_eq!(shorts.len(), 4);
    }

    #[test]
    fn test_05() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(-605, 1513),
            bm: IntPoint::new(-1010, 207),
            b: IntPoint::new(1024, 1024),
        };

        let shorts = spline.approximate(normalize_unit_value(0.8), 5);
        assert_eq!(shorts.len(), 10);
    }
}
