use crate::int::collision::collider::Collider;
use crate::int::collision::solver::x_segment::XOverlap;
use crate::int::math::x_segment::XSegment;
use alloc::vec::Vec;
use core::mem::swap;
use core::ops::Range;
use crate::int::base::spline::IntSpline;
use crate::int::collision::approximation::SplineApproximation;
use crate::int::collision::fit::{FIT_32_ATOM, FIT_32_MAX};

struct Base {
    generation: u32,
    segment: XSegment,
}

struct Mark {
    primary: Base,
    secondary: Base,
    overlap: XOverlap,
}

#[derive(Clone)]
struct XBox {
    generation: u32,
    collider: Collider,
}

#[derive(Clone)]
struct Pair {
    primary: XBox,
    secondary: XBox,
}

pub struct Solver {
    list: Vec<Pair>,
    next: Vec<Pair>,
    marks: Vec<Mark>,
}

impl Solver {
    #[inline]
    fn intersect(&mut self, primary: Collider, secondary: Collider) {
        self.marks.clear();
        self.list.clear();

        let max_log_size = primary.boundary.max_log_size().max(secondary.boundary.max_log_size());
        let min_iter_count = if max_log_size > FIT_32_MAX {
            max_log_size - FIT_32_MAX
        } else {
            0
        };
        let max_iter_count = max_log_size - FIT_32_ATOM;

        let x0 = XBox { generation: 0, collider: primary };
        let x1 = XBox { generation: 0, collider: secondary };

        self.list.push(Pair { primary: x0, secondary: x1 });

        self.box_cross(min_iter_count..max_iter_count);

        if self.list.is_empty() {
            return;
        }

        self.segment_cross();
    }

    #[inline]
    fn box_cross(&mut self, iter_range: Range<u32>) {
        let mut generation = 0;

        while !self.list.is_empty() && iter_range.contains(&generation) && self.list.len() <= 1024 {
            for pair in self.list.iter() {
                if pair.overlap() {
                    pair.split_into(&mut self.next);
                }
            }

            swap(&mut self.next, &mut self.list);

            self.next.clear();
            generation += 1;
        }
    }

    #[inline]
    fn segment_cross(&mut self) {
        for pair in self.list.iter() {
            let s0 = pair.primary.collider.to_segment();
            let s1 = pair.secondary.collider.to_segment();

            if let Some(overlap) = s0.cross(&s1) {
                self.marks.push(Mark {
                    primary: Base {
                        generation: pair.primary.generation,
                        segment: s0,
                    },
                    secondary: Base {
                        generation: pair.secondary.generation,
                        segment: s1,
                    },
                    overlap,
                });
            }
        }
    }
}

impl Pair {
    #[inline]
    fn is_boundary_overlap(&self) -> bool {
        self.primary
            .collider
            .boundary
            .is_intersect_border_include(&self.secondary.collider.boundary)
    }

    #[inline]
    fn overlap(&self) -> bool {
        self.primary.collider.overlap(&self.secondary.collider)
    }

    #[inline]
    fn split_into(&self, vec: &mut Vec<Pair>) {
        let s0 = self.primary.collider.split();
        let s1 = self.secondary.collider.split();

        match (s0, s1) {
            (Some((c0, c1)), Some((c2, c3))) => {
                let g0 = self.primary.generation + 1;
                let g1 = self.secondary.generation + 1;
                let x0 = XBox { generation: g0, collider: c0 };
                let x1 = XBox { generation: g0, collider: c1 };
                let x2 = XBox { generation: g1, collider: c2 };
                let x3 = XBox { generation: g1, collider: c3 };

                vec.push(Pair { primary: x0.clone(), secondary: x2.clone() });
                vec.push(Pair { primary: x0, secondary: x3.clone() });
                vec.push(Pair { primary: x1.clone(), secondary: x2 });
                vec.push(Pair { primary: x1, secondary: x3 });
            }
            (Some((c0, c1)), None) => {
                let g0 = self.primary.generation + 1;
                let x0 = XBox { generation: g0, collider: c0 };
                let x1 = XBox { generation: g0, collider: c1 };

                vec.push(Pair { primary: x0, secondary: self.secondary.clone() });
                vec.push(Pair { primary: x1, secondary: self.secondary.clone() });
            }
            (None, Some((c2, c3))) => {
                let g1 = self.secondary.generation + 1;
                let x2 = XBox { generation: g1, collider: c2 };
                let x3 = XBox { generation: g1, collider: c3 };

                vec.push(Pair { primary: self.primary.clone(), secondary: x2 });
                vec.push(Pair { primary: self.primary.clone(), secondary: x3 });
            }
            (None, None) => vec.push(self.clone())
        }
    }
}

// debug api

pub trait SplineOverlay {
    fn overlay(&self, other: &Self) -> Vec<XOverlap>;
}

impl SplineOverlay for IntSpline {
    fn overlay(&self, other: &Self) -> Vec<XOverlap> {
        let mut solver = Solver::default();
        solver.intersect(self.collider(), other.collider());
        solver.marks.iter().map(|m| m.overlap).collect()
    }
}

impl IntSpline {
    fn collider(&self) -> Collider {
        match self {
            IntSpline::Arc(s) => s.clone().into_collider(),
            IntSpline::Line(s) => s.clone().into_collider(),
            IntSpline::Square(s) => s.clone().into_collider(),
            IntSpline::Cubic(s) => s.clone().into_collider(),
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            list: Vec::with_capacity(8),
            next: Vec::with_capacity(8),
            marks: Vec::with_capacity(16),
        }
    }
}

trait SortByX {
    fn sort_by_x(&mut self);
}

impl SortByX for [XBox] {
    #[inline]
    fn sort_by_x(&mut self) {
        self.sort_unstable_by(|a, b| a.collider.boundary.min.x.cmp(&b.collider.boundary.min.x));
    }
}

impl XBox {
    #[inline(always)]
    fn is_not_overlap_by_y(&self, other: &Self) -> bool {
        let min_y0 = self.collider.boundary.min.y;
        let max_y0 = self.collider.boundary.max.y;

        let min_y1 = other.collider.boundary.min.y;
        let max_y1 = other.collider.boundary.max.y;

        min_y0 > max_y1 || max_y0 < min_y1
    }
}

#[cfg(test)]
mod tests {
    use crate::int::base::spline::IntSpline;
    use crate::int::bezier::spline_cubic::IntCubicSpline;
    use crate::int::collision::approximation::SplineApproximation;
    use crate::int::collision::solver::cross::{Pair, SplineOverlay, XBox};
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_0() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(0, 0),
                IntPoint::new(0, 50),
                IntPoint::new(50, 100),
                IntPoint::new(100, 100),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(50, 0),
                IntPoint::new(50, 50),
                IntPoint::new(0, 100),
                IntPoint::new(-50, 100),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_1() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(0, -100),
                IntPoint::new(413, 295),
                IntPoint::new(100, 0),
                IntPoint::new(-200, 351),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(100, 100),
                IntPoint::new(100, 200),
                IntPoint::new(200, 100),
                IntPoint::new(200, 200),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_2() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(167, 141),
                IntPoint::new(103, 161),
                IntPoint::new(-50, 175),
                IntPoint::new(-200, 351),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(150, 150),
                IntPoint::new(175, 150),
                IntPoint::new(200, 150),
                IntPoint::new(200, 200),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_3() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(167, 141),
                IntPoint::new(135, 151),
                IntPoint::new(80, 159),
                IntPoint::new(15, 187),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(115, 143),
                IntPoint::new(124, 150),
                IntPoint::new(137, 150),
                IntPoint::new(150, 150),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_4() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(167, 141),
                IntPoint::new(135, 151),
                IntPoint::new(80, 159),
                IntPoint::new(15, 187),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(115, 143),
                IntPoint::new(124, 150),
                IntPoint::new(137, 150),
                IntPoint::new(150, 150),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_5() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(167, 141),
                IntPoint::new(151, 146),
                IntPoint::new(129, 150),
                IntPoint::new(103, 157),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(130, 149),
                IntPoint::new(136, 150),
                IntPoint::new(143, 150),
                IntPoint::new(150, 150),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_6() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(138, 147),
                IntPoint::new(128, 150),
                IntPoint::new(116, 153),
                IntPoint::new(103, 157),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(130, 149),
                IntPoint::new(133, 149),
                IntPoint::new(136, 149),
                IntPoint::new(139, 149),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_7() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(167, 141),
                IntPoint::new(151, 146),
                IntPoint::new(129, 150),
                IntPoint::new(103, 157),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(130, 149),
                IntPoint::new(136, 150),
                IntPoint::new(143, 150),
                IntPoint::new(150, 150),
            ],
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b));

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_pair_0() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(167, 141),
                IntPoint::new(103, 161),
                IntPoint::new(-50, 175),
                IntPoint::new(-200, 351),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(150, 150),
                IntPoint::new(175, 150),
                IntPoint::new(200, 150),
                IntPoint::new(200, 200),
            ],
        };

        let x0 = XBox { generation: 0, collider: a.into_collider() };
        let x1 = XBox { generation: 0, collider: b.into_collider() };
        let pair = Pair { primary: x0, secondary: x1 };

        assert_eq!(pair.overlap(), false);
    }

    #[test]
    fn test_pair_1() {
        let a = IntCubicSpline {
            anchors: [
                IntPoint::new(167, 141),
                IntPoint::new(151, 146),
                IntPoint::new(129, 150),
                IntPoint::new(103, 157),
            ],
        };

        let b = IntCubicSpline {
            anchors: [
                IntPoint::new(130, 149),
                IntPoint::new(136, 150),
                IntPoint::new(143, 150),
                IntPoint::new(150, 150),
            ],
        };

        let x0 = XBox { generation: 0, collider: a.into_collider() };
        let x1 = XBox { generation: 0, collider: b.into_collider() };
        let pair = Pair { primary: x0, secondary: x1 };

        assert_eq!(pair.overlap(), true);
    }
}
