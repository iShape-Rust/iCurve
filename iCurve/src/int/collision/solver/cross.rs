use crate::data::bitset::IndexBitSet;
use crate::data::four_vec::FourVec;
use crate::int::collision::collider::Collider;
use crate::int::collision::colliding::Colliding;
use crate::int::collision::spline::Spline;
use crate::int::math::point::IntPoint;
use alloc::vec::Vec;
use core::mem::swap;

struct Box {
    is_primary: bool,
    collider: Collider,
}

pub struct Solver {
    list: Vec<Box>,
    next: Vec<Box>,
    bitset: IndexBitSet,
}

impl Solver {

    #[inline]
    fn intersect(&mut self, primary: Collider, secondary: Collider) -> Option<Vec<Spline>> {
        if !primary.boundary.is_intersect_border_include(&secondary.boundary) {
            return None;
        }

        let max_log_size = primary.max_log_size().max(secondary.max_log_size());

        if max_log_size.is_atom() {
            return self.segment_cross();
        }
        self.list.clear();
        self.list.push(Box { is_primary: true, collider: primary });
        self.list.push(Box { is_primary: false, collider: secondary });

        let min_iter_count = if max_log_size > FIT_32_MAX { max_log_size - FIT_32_MAX } else { 0 };
        let max_iter_count = max_log_size - FIT_32_ATOM;
        let iter_range = min_iter_count..max_iter_count;

        let mut cxi: Option<FourVec<IntPoint>>;
        let mut generation = 0;

        while !self.list.is_empty() && iter_range.contains(&generation) && self.list.len() <= 1024 {
            self.list.sort_by_x();

            for (i, bi) in self.list[0..self.list.len() - 1].iter().enumerate() {
                cxi = None;
                let mut primary_splitted = false;
                let max_i = bi.collider.boundary.max;
                for (j, bj) in self.list[i + 1..].iter().enumerate() {
                    if max_i < bi.collider.boundary.min {
                        break;
                    }

                    if bi.is_primary != bj.is_primary || bi.is_not_overlap_by_y(&bj) {
                        continue;
                    }

                    if primary_splitted && self.bitset.contains(j) {
                        continue;
                    }

                    let mi = bi.collider.max_log_size();
                    let mj = bj.collider.max_log_size();

                    let is_overlap = if mi.is_fit32() && mj.is_fit32() {
                        if mi.is_atom() && mj.is_atom() {
                            bi.collider
                                .spline
                                .as_x_segment()
                                .overlap(&bj.collider.spline.as_x_segment())
                        } else {
                            let cx = cxi.unwrap_or_else(|| bi.collider.spline.convex_hull());
                            let cxj = bj.collider.spline.convex_hull();

                            let is_overlap = cx.overlap(&cxj);

                            cxi = Some(cx);

                            is_overlap
                        }
                    } else {
                        true
                    };

                    if !is_overlap {
                        continue;
                    }

                    if !primary_splitted {
                        primary_splitted = true;
                        let (c0, c1) = bi.collider.spline.bisect();
                        self.next.push(Box { is_primary: true, collider: c0 });
                        self.next.push(Box { is_primary: true, collider: c1 });
                    }

                    let (c0, c1) = bj.collider.spline.bisect();
                    self.next.push(Box { is_primary: false, collider: c0 });
                    self.next.push(Box { is_primary: false, collider: c1 });
                    self.bitset.insert(i + j + 1);
                }
            }

            swap(&mut self.next, &mut self.list);
            self.bitset.clear();
            generation += 1;
        }

        if self.list.is_empty() {
            return None;
        }

        None
    }

    fn segment_cross(&mut self) -> Option<Vec<Spline>> {
        for (i, bi) in self.list[0..self.list.len() - 1].iter().enumerate() {
            let si = bi.collider.spline.as_x_segment();
            for bj in self.list[i + 1..].iter() {
                let sj = bj.collider.spline.as_x_segment();
                if si.b.x < sj.a.x {
                    break;
                }

                if bi.is_primary != bj.is_primary || !si.overlap(&sj) {
                    continue;
                }



            }
        }

        None
    }


}

impl Default for Solver {
    fn default() -> Self {
        Self {
            list: Vec::with_capacity(16),
            next: Vec::with_capacity(16),
            bitset: IndexBitSet::with_size(256),
        }
    }
}

trait SortByX {
    fn sort_by_x(&mut self);
}

impl SortByX for [Box] {
    #[inline]
    fn sort_by_x(&mut self) {
        self.sort_unstable_by(|a, b| a.collider.boundary.min.x.cmp(&b.collider.boundary.min.x));
    }
}

impl Box {
    #[inline(always)]
    fn is_not_overlap_by_y(&self, other: &Self) -> bool {
        let min_y0 = self.collider.boundary.min.y;
        let max_y0 = self.collider.boundary.max.y;

        let min_y1 = other.collider.boundary.min.y;
        let max_y1 = other.collider.boundary.max.y;

        min_y0 > max_y1 || max_y0 < min_y1
    }
}

impl Collider {
    #[inline(always)]
    fn max_log_size(&self) -> u32 {
        self.boundary.width().max(self.boundary.height()).ilog2()
    }
}

const FIT_32_MAX: u32 = 28;
const FIT_32_ATOM: u32 = 4;

trait Fit32 {
    fn is_fit32(&self) -> bool;
    fn is_atom(&self) -> bool;
}

impl Fit32 for u32 {
    #[inline(always)]
    fn is_fit32(&self) -> bool {
        *self < FIT_32_MAX
    }

    #[inline(always)]
    fn is_atom(&self) -> bool {
        *self < FIT_32_ATOM
    }
}

#[cfg(test)]
mod tests {
    use crate::int::bezier::spline_cube::IntCubeSpline;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_0() {
        let a = IntCubeSpline {
            anchors: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 50),
                IntPoint::new(50, 100),
                IntPoint::new(100, 100),
            ],
        };

        let b = IntCubeSpline {
            anchors: [
                IntPoint::new(50, 0),
                IntPoint::new(50, 50),
                IntPoint::new(0, 100),
                IntPoint::new(-50, 100),
            ],
        };
    }
}
