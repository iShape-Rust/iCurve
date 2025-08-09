use alloc::vec::Vec;
use crate::int::collision::collider::Collider;
use crate::int::collision::space::Space;

#[derive(Clone)]
pub(super) struct XBox {
    pub(super) generation: u32,
    pub(super) collider: Collider,
}

#[derive(Clone)]
pub(super) struct Pair {
    pub(super) primary: XBox,
    pub(super) secondary: XBox,
}

impl Pair {
    #[inline]
    pub(super) fn overlap(&self, space: &Space) -> bool {
        self.primary.collider.overlap(&self.secondary.collider, space)
    }

    #[inline]
    pub(super) fn split_into(&self, space: &Space, vec: &mut Vec<Pair>) {
        let s0 = self.primary.collider.split(space);
        let s1 = self.secondary.collider.split(space);

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

#[cfg(test)]
mod tests {
    use crate::int::bezier::spline_cubic::IntCubicSpline;
    use crate::int::collision::approximation::SplineApproximation;
    use crate::int::collision::pair::{Pair, XBox};
    use crate::int::collision::space::Space;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_pair_0() {
        let space = Space::default();

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

        let x0 = XBox { generation: 0, collider: a.into_collider(&space) };
        let x1 = XBox { generation: 0, collider: b.into_collider(&space) };
        let pair = Pair { primary: x0, secondary: x1 };

        let overlap = pair.overlap(&space);

        assert_eq!(overlap, true);
    }

    #[test]
    fn test_1() {
        let space = Space::default();

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

        let x0 = XBox { generation: 0, collider: a.into_collider(&space) };
        let x1 = XBox { generation: 0, collider: b.into_collider(&space) };
        let pair = Pair { primary: x0, secondary: x1 };

        assert_eq!(pair.overlap(&space), true);
    }
}
