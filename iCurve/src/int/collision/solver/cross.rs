use crate::data::bitset::IndexBitSet;
use crate::data::four_vec::FourVec;
use crate::int::collision::collider::Collider;
use crate::int::collision::colliding::Colliding;
use crate::int::collision::solver::x_segment::XOverlap;
use crate::int::collision::spline::Spline;
use crate::int::math::point::IntPoint;
use crate::int::math::x_segment::XSegment;
use alloc::vec::Vec;
use core::mem::swap;
use core::ops::Range;

struct Base {
    generation: u32,
    segment: XSegment,
}

struct Mark {
    primary: Base,
    secondary: Base,
    overlap: XOverlap,
}

struct Box {
    generation: u32,
    collider: Collider,
}

pub struct Solver {
    primary_list: Vec<Box>,
    primary_next: Vec<Box>,
    secondary_list: Vec<Box>,
    secondary_next: Vec<Box>,
    secondary_splitted_bitset: IndexBitSet,
    marks: Vec<Mark>,
}

impl Solver {
    #[inline]
    fn intersect(&mut self, primary: Collider, secondary: Collider) {
        self.marks.clear();

        if !primary
            .boundary
            .is_intersect_border_include(&secondary.boundary)
        {
            return;
        }

        let max_log_size = primary.max_log_size().max(secondary.max_log_size());

        self.primary_list.clear();
        self.secondary_list.clear();

        self.primary_list.push(Box {
            generation: 0,
            collider: primary,
        });

        self.secondary_list.push(Box {
            generation: 0,
            collider: secondary,
        });

        let min_iter_count = if max_log_size > FIT_32_MAX {
            max_log_size - FIT_32_MAX
        } else {
            0
        };
        let max_iter_count = max_log_size - FIT_32_ATOM;

        self.box_cross(min_iter_count..max_iter_count);

        if self.primary_list.is_empty() {
            return;
        }

        self.segment_cross();
    }

    #[inline]
    fn box_cross(&mut self, iter_range: Range<u32>) {
        let mut cxi: Option<FourVec<IntPoint>>;
        let mut generation = 0;

        while !self.primary_list.is_empty() && iter_range.contains(&generation) && self.primary_list.len() <= 1024 {
            self.primary_list.sort_by_x();
            self.secondary_list.sort_by_x();
            let mut j0 = 0;

            for bi in self.primary_list.iter() {
                cxi = None;
                let mut primary_splitted = false;
                let min_xi = bi.collider.boundary.min.x;
                let max_xi = bi.collider.boundary.max.x;

                // scroll to first overlap
                while j0 < self.secondary_list.len() {
                    let max_xj = self.secondary_list[j0].collider.boundary.max.x;
                    if min_xi <= max_xj {
                        break;
                    }
                    j0 += 1;
                }

                for (j, bj) in self.secondary_list[j0..].iter().enumerate() {
                    let min_xj = bj.collider.boundary.min.x;
                    if min_xj > max_xi {
                        break;
                    }

                    if bi.is_not_overlap_by_y(&bj) {
                        continue;
                    }
                    let index = j0 + j;

                    let secondary_splitted = self.secondary_splitted_bitset.contains(index);
                    if primary_splitted && secondary_splitted {
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
                        self.primary_next.push(Box {
                            generation: bi.generation + 1,
                            collider: c0,
                        });
                        self.primary_next.push(Box {
                            generation: bi.generation + 1,
                            collider: c1,
                        });
                    }

                    let (c0, c1) = bj.collider.spline.bisect();
                    self.secondary_next.push(Box {
                        generation: bj.generation + 1,
                        collider: c0,
                    });
                    self.secondary_next.push(Box {
                        generation: bj.generation + 1,
                        collider: c1,
                    });
                    self.secondary_splitted_bitset.insert(index);
                }
            }

            swap(&mut self.primary_next, &mut self.primary_list);
            swap(&mut self.secondary_next, &mut self.secondary_list);

            self.primary_next.clear();
            self.secondary_next.clear();
            self.secondary_splitted_bitset.clear();
            generation += 1;
        }
    }

    #[inline]
    fn segment_cross(&mut self) {
        let mut j0 = 0;

        self.primary_list.sort_by_x();
        self.secondary_list.sort_by_x();
        
        for bi in self.primary_list.iter() {
            let si = bi.collider.spline.as_x_segment();

            // scroll to first overlap
            while j0 < self.secondary_list.len() {
                let max_xj = self.secondary_list[j0].collider.boundary.max.x;
                if si.a.x <= max_xj {
                    break;
                }
                j0 += 1;
            }

            for bj in self.secondary_list[j0..].iter() {
                let sj = bj.collider.spline.as_x_segment();
                if sj.a.x > si.b.x {
                    break;
                }

                if !si.is_overlap_y(&sj) {
                    continue;
                }

                if let Some(result) = si.cross(&sj) {
                    self.marks.push(Mark {
                        primary: Base {
                            generation: bi.generation,
                            segment: si,
                        },
                        secondary: Base {
                            generation: bj.generation,
                            segment: sj,
                        },
                        overlap: result,
                    });
                }
            }
        }
    }
}

// debug api

pub trait SplineOverlay {
    fn overlay(&self, other: &Self) -> Vec<XOverlap>;
}

impl SplineOverlay for Spline {
    fn overlay(&self, other: &Self) -> Vec<XOverlap> {
        let mut solver = Solver::default();
        let c0 = Collider {
            boundary: self.boundary(),
            spline: self.clone(),
        };
        let c1 = Collider {
            boundary: other.boundary(),
            spline: other.clone(),
        };
        solver.intersect(c0, c1);

        solver.marks.iter().map(|m| m.overlap).collect()
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            primary_list: Vec::with_capacity(8),
            primary_next: Vec::with_capacity(8),
            secondary_list: Vec::with_capacity(8),
            secondary_next: Vec::with_capacity(8),
            secondary_splitted_bitset: IndexBitSet::with_size(256),
            marks: Vec::with_capacity(16),
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
    use crate::int::collision::solver::cross::SplineOverlay;
    use crate::int::collision::spline::Spline;
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

        let result = Spline::Cube(a).overlay(&Spline::Cube(b));

        assert_eq!(result.len(), 1);
    }
}
