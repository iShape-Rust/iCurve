use crate::data::link_list::{LinkList, EMPTY_REF};
use crate::int::bezier::spline::IntCADSpline;
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
        debug_assert!(min_cos <= 1024);
        Solver::approximate(self, min_cos, min_len)
    }
}

pub(super) struct Solver<'a, Spline> {
    min_cos: i64,
    st_dir: IntPoint,
    ed_dir: IntPoint,
    min_len_power: u32,
    spline: &'a Spline,
    segments: LinkList<IntShort>,
}

impl<'a, Spline: IntCADSpline> Solver<'a, Spline> {
    #[inline]
    pub(super) fn approximate(spline: &Spline, min_cos: u32, min_len: u32) -> Vec<IntShort> {
        let min_len_power = min_len.ilog2();
        let st_dir = spline.start_dir();
        let ed_dir = spline.end_dir();

        let segments = LinkList::new(vec![IntShort {
            step: 0,
            split_factor: 0,
            dir: (spline.end() - spline.start()).normalized_10bit(),
            a: spline.start(),
            b: spline.end(),
        }]);

        Solver {
            min_cos: (min_cos << 10) as i64,
            st_dir,
            ed_dir,
            min_len_power,
            spline,
            segments,
        }
            .process()
    }

    #[inline]
    fn process(&mut self) -> Vec<IntShort> {
        let mut buffer = Vec::with_capacity(16);
        buffer.push(0);

        let mut to_split = Vec::with_capacity(16);

        while !buffer.is_empty() {
            for &index in buffer.iter() {
                if self.split_test(index) {
                    to_split.push(index);
                }
            }

            buffer.clear();
            for &index in to_split.iter() {
                self.split(index, &mut buffer);
            }
            to_split.clear();
        }

        let mut shorts = Vec::with_capacity(self.segments.len());
        let mut index = 0;
        while index != EMPTY_REF {
            let node = self.segments.get(index);
            shorts.push(node.item);
            index = node.next
        }

        shorts
    }

    fn split_test(&self, index: u32) -> bool {
        let node = self.segments.get(index);
        let prev = node.prev;
        let next = node.next;
        let dir = node.item.dir;
        let prev_dir = if prev != EMPTY_REF {
            self.segments.get(prev).item.dir
        } else {
            self.st_dir
        };

        let prev_dot_product = dir.dot_product(&prev_dir);
        if prev_dot_product < self.min_cos {
            return true;
        }

        let next_dir = if next != EMPTY_REF {
            self.segments.get(next).item.dir
        } else {
            self.ed_dir
        };

        let next_dot_product = dir.dot_product(&next_dir);
        next_dot_product < self.min_cos
    }

    fn split(&mut self, index: u32, result: &mut Vec<u32>) {
        let short = self.segments.get(index).item;

        let split_factor = short.split_factor + 1;
        let m = self.spline.split_at(short.step + 1, split_factor);
        let ma = m - short.a;
        let bm = short.b - m;

        if short.a == m || short.b == m {
            // split is not possible
            return;
        }

        let s0 = IntShort {
            step: short.step << 1,
            split_factor,
            dir: ma.normalized_10bit(),
            a: short.a,
            b: m,
        };

        let s1 = IntShort {
            step: (short.step + 1) << 1,
            split_factor,
            dir: bm.normalized_10bit(),
            a: m,
            b: short.b,
        };

        let (i0, i1) = self.segments.split_at(index, s0, s1);

        if !ma.is_small(self.min_len_power) {
            result.push(i0)
        }

        if !bm.is_small(self.min_len_power) {
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
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_00() {
        let p = IntPoint::new(100, 100);

        assert_eq!(p.is_small(2), false);
        assert_eq!(p.is_small(3), false);
        assert_eq!(p.is_small(4), false);
        assert_eq!(p.is_small(5), false);
        assert_eq!(p.is_small(6), false);
        assert_eq!(p.is_small(7), false);
        assert_eq!(p.is_small(8), true);
    }

    #[test]
    fn test_01() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 50),
            bm: IntPoint::new(100, 50),
            b: IntPoint::new(100, 0),
        };

        let shorts = spline.approximate(800, 8);
        assert_eq!(shorts.len(), 6);
    }

    #[test]
    fn test_02() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 50),
            bm: IntPoint::new(100, 50),
            b: IntPoint::new(100, 0),
        };

        let shorts = spline.approximate(800, 32);
        assert_eq!(shorts.len(), 5);
    }

    #[test]
    fn test_03() {
        let spline = IntCubeSpline {
            a: IntPoint::new(0, 0),
            am: IntPoint::new(0, 50),
            bm: IntPoint::new(100, 50),
            b: IntPoint::new(100, 0),
        };

        let shorts = spline.approximate(900, 4);
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

        let shorts = spline.approximate(900, 16);
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

        let shorts = spline.approximate(800, 5);
        assert_eq!(shorts.len(), 10);
    }
}
