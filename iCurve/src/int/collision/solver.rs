use crate::int::base::spline::IntSpline;
use crate::int::bezier::spline::SplitPosition;
use crate::int::collision::approximation::SplineApproximation;
use crate::int::collision::collider::Collider;
use crate::int::collision::pair::{Pair, XBox};
use crate::int::collision::space::Space;
use crate::int::collision::x_segment::XOverlap;
use crate::int::math::point::IntPoint;
use crate::int::math::x_segment::XSegment;
use alloc::vec::Vec;
use core::mem::swap;
use core::ops::Range;

struct MarkPoint {
    pub(super) point: IntPoint,
    pub(super) a_pos: SplitPosition,
    pub(super) b_pos: SplitPosition,
}

pub(super) struct Mark {
    pub(super) p0: MarkPoint,
    pub(super) p1: Option<MarkPoint>,
}

pub struct Solver {
    list: Vec<Pair>,
    next: Vec<Pair>,
    pub(super) marks: Vec<Mark>,
    space: Space,
}

impl Solver {
    #[inline]
    fn intersect(&mut self, a_col: Collider, b_col: Collider) {
        self.marks.clear();
        self.list.clear();

        let max_log_size = a_col.size_level.max(b_col.size_level);
        let min_iter_count = if max_log_size > self.space.convex_level {
            max_log_size - self.space.convex_level
        } else {
            0
        };
        let max_iter_count = max_log_size - self.space.line_level;

        let a_box = XBox {
            position: SplitPosition { power: 0, value: 0 },
            collider: a_col,
        };
        let b_box = XBox {
            position: SplitPosition { power: 0, value: 0 },
            collider: b_col,
        };

        self.list.push(Pair { a_box, b_box });

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
            let sa = pair.a_box.collider.to_segment();
            let sb = pair.b_box.collider.to_segment();

            let sxa: XSegment = sa.into();
            let sxb: XSegment = sb.into();

            if let Some(overlap) = sxa.cross(&sxb) {
                match overlap {
                    XOverlap::Point(point) => {
                        let a_pos = sa.position(point);
                        let b_pos = sb.position(point);
                        let p0 = MarkPoint {
                            point,
                            a_pos,
                            b_pos,
                        };
                        self.marks.push(Mark { p0, p1: None });
                    }
                    XOverlap::Segment(s) => {
                        let a_pos_0 = sa.position(s[0]);
                        let b_pos_0 = sb.position(s[0]);
                        let p0 = MarkPoint {
                            point: s[0],
                            a_pos: a_pos_0,
                            b_pos: b_pos_0,
                        };
                        let a_pos_1 = sa.position(s[1]);
                        let b_pos_1 = sb.position(s[1]);
                        let p1 = MarkPoint {
                            point: s[1],
                            a_pos: a_pos_1,
                            b_pos: b_pos_1,
                        };
                        self.marks.push(Mark { p0, p1: Some(p1) });
                    }
                }
            }
        }
    }
}

trait SegmentPosition {
    fn position(&self, p: IntPoint) -> SplitPosition;
}

impl SegmentPosition for [IntPoint; 2] {
    #[inline]
    fn position(&self, p: IntPoint) -> SplitPosition {
        let dx = (self[0].x - self[1].x).unsigned_abs();
        let dy = (self[0].y - self[1].y).unsigned_abs();

        let (l, t) = if dx >= dy {
            (dx, (self[0].x - p.x).unsigned_abs())
        } else {
            (dy, (self[0].y - p.y).unsigned_abs())
        };
        let power = l.ilog2();
        let s = t.max(l - 1);
        let value = (s << power) / l;

        SplitPosition { power, value }
    }
}

pub trait SplineOverlay {
    fn overlay(&self, other: &Self, space: &Space) -> Vec<XOverlap>;
}

impl SplineOverlay for IntSpline {
    fn overlay(&self, other: &Self, space: &Space) -> Vec<XOverlap> {
        let mut solver = Solver::default();
        solver.intersect(self.collider(space), other.collider(space));
        solver
            .marks
            .iter()
            .map(|m| {
                if let Some(p1) = &m.p1 {
                    XOverlap::Segment([m.p0.point, p1.point])
                } else {
                    XOverlap::Point(m.p0.point)
                }
            })
            .collect()
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
    use crate::int::base::spline::IntSpline;
    use crate::int::bezier::spline_cubic::IntCubicSpline;
    use crate::int::collision::approximation::SplineApproximation;
    use crate::int::collision::solver::{Solver, SplineOverlay};
    use crate::int::collision::space::Space;
    use crate::int::math::point::IntPoint;
    use core::f64::consts::PI;

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
    fn test_5() {
        let a = IntCubicSpline {
            anchors: [[0, 0], [0, 1000], [2000, 1000], [2000, 0]].map(|p| p.into()),
        };

        let b = IntCubicSpline {
            anchors: [[0, 20], [0, 1020], [2000, 1020], [2000, 20]].map(|p| p.into()),
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b), &Space::default());

        assert!(result.len() > 0);
    }

    #[test]
    fn test_6() {
        let a = IntCubicSpline {
            anchors: [[0, 0], [0, 100_000], [200_000, 100_000], [200_000, 0]].map(|p| p.into()),
        };

        let b = IntCubicSpline {
            anchors: [[0, 20], [0, 100_020], [200_000, 100_020], [200_000, 20]].map(|p| p.into()),
        };

        let result = IntSpline::Cubic(a).overlay(&IntSpline::Cubic(b), &Space::default());

        assert!(result.len() > 0);
    }

    #[test]
    fn test_random_0() {
        const COUNT: usize = 80;
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

                        solver.intersect(
                            a.clone().into_collider(&solver.space),
                            b.clone().into_collider(&solver.space),
                        );
                        solver.marks.is_empty();

                        if solver.marks.is_empty() {
                            panic!("Can not be empty a: {:?}, b: {:?}", &a.anchors, &b.anchors);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_random_1() {
        const COUNT: usize = 80;
        const R: i64 = 10_000_000;
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

                        solver.intersect(
                            a.clone().into_collider(&solver.space),
                            b.clone().into_collider(&solver.space),
                        );
                        solver.marks.is_empty();

                        if solver.marks.is_empty() {
                            panic!("Can not be empty a: {:?}, b: {:?}", &a.anchors, &b.anchors);
                        }
                    }
                }
            }
        }
    }
}
