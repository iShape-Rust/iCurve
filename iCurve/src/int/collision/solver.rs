use crate::int::collision::pair::{Pair, XBox};
use crate::int::math::x_segment::XSegment;
use alloc::vec::Vec;
use core::mem::swap;
use core::ops::Range;
use crate::int::base::spline::IntSpline;
use crate::int::collision::approximation::SplineApproximation;
use crate::int::collision::collider::Collider;
use crate::int::collision::space::Space;
use crate::int::collision::x_segment::XOverlap;

struct Base {
    generation: u32,
    segment: XSegment,
}

struct Mark {
    primary: Base,
    secondary: Base,
    overlap: XOverlap,
}

pub struct Solver {
    list: Vec<Pair>,
    next: Vec<Pair>,
    marks: Vec<Mark>,
    space: Space
}

impl Solver {
    #[inline]
    fn intersect(&mut self, primary: Collider, secondary: Collider) {
        self.marks.clear();
        self.list.clear();

        let max_log_size = primary.size_level.max(secondary.size_level);
        let min_iter_count = if max_log_size > self.space.convex_level {
            max_log_size - self.space.convex_level
        } else {
            0
        };
        let max_iter_count = max_log_size - self.space.line_level;

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
                if pair.overlap(&self.space) {
                    pair.split_into(&self.space, &mut self.next);
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

pub trait SplineOverlay {
    fn overlay(&self, other: &Self, space: &Space) -> Vec<XOverlap>;
}

impl SplineOverlay for IntSpline {
    fn overlay(&self, other: &Self, space: &Space) -> Vec<XOverlap> {
        let mut solver = Solver::default();
        solver.intersect(self.collider(space), other.collider(space));
        solver.marks.iter().map(|m| m.overlap).collect()
    }
}

impl IntSpline {
    fn collider(&self, space: &Space) -> Collider {
        match self {
            IntSpline::Arc(s) => s.clone().into_collider(space),
            IntSpline::Line(s) => s.clone().into_collider(space),
            IntSpline::Square(s) => s.clone().into_collider(space),
            IntSpline::Cubic(s) => s.clone().into_collider(space),
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            list: Vec::with_capacity(8),
            next: Vec::with_capacity(8),
            marks: Vec::with_capacity(16),
            space: Space::with_line_level(4),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;
    use crate::int::base::spline::IntSpline;
    use crate::int::bezier::spline_cubic::IntCubicSpline;
    use crate::int::collision::approximation::SplineApproximation;
    use crate::int::collision::solver::{Solver, SplineOverlay};
    use crate::int::collision::space::Space;
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

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b), &Space::default());

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

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b), &Space::default());

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

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b), &Space::default());

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

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b), &Space::default());

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

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b), &Space::default());

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_random_0() {
        const COUNT: usize = 20;
        const R: i64 = 10_000;
        const F: f64 = R as f64;
        let mut pnt_offset = [IntPoint::zero(); COUNT];

        let delta_angle = 2.0 * PI / COUNT as f64;
        let mut angle = 0.0f64;
        for p in pnt_offset.iter_mut() {
            let (sn, cs) = angle.sin_cos();
            p.x = (F * cs) as i64;
            p.y = (F * sn) as i64;
            angle += delta_angle;
        }

        let mut a = IntCubicSpline {
            anchors: [
                IntPoint::new(-R, 0),
                IntPoint::new(0, 0),
                IntPoint::new(0, 0),
                IntPoint::new(R, 0),
            ],
        };

        let mut b = IntCubicSpline {
            anchors: [
                IntPoint::new(0, -R),
                IntPoint::new(0, 0),
                IntPoint::new(0, 0),
                IntPoint::new(0, R),
            ],
        };

        let mut solver = Solver::default();

        for &p0 in pnt_offset.iter() {
            a.anchors[1] = b.anchors[0] + p0;
            for &p1 in pnt_offset.iter() {
                a.anchors[2] = b.anchors[3] + p1;
                for &p2 in pnt_offset.iter() {
                    b.anchors[1] = b.anchors[0] + p2;
                    for &p3 in pnt_offset.iter() {
                        b.anchors[2] = b.anchors[3] + p3;

                        solver.intersect(a.clone().into_collider(&solver.space), b.clone().into_collider(&solver.space));
                        solver.marks.is_empty();

                        if solver.marks.is_empty() {
                            panic!("Can not be empty");
                        }
                    }
                }
            }
        }
    }
}