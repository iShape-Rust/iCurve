use crate::int::bezier::spline::SplitPosition;
use crate::int::collision::collider::Collider;
use crate::int::collision::space::Space;
use alloc::vec::Vec;

#[derive(Clone)]
pub(super) struct XBox {
    pub(super) position: SplitPosition,
    pub(super) collider: Collider,
}

#[derive(Clone)]
pub(super) struct Pair {
    pub(super) a_box: XBox,
    pub(super) b_box: XBox,
}

impl Pair {
    #[inline]
    pub(super) fn overlap(&self, space: &Space) -> bool {
        self.a_box.collider.overlap(&self.b_box.collider, space)
    }

    #[inline]
    pub(super) fn split_into(&self, space: &Space, vec: &mut Vec<Pair>) {
        let a = self.a_box.bisect(space);
        let b = self.b_box.bisect(space);

        match (a, b) {
            (Some((a0, a1)), Some((b0, b1))) => {
                vec.push(Pair {
                    a_box: a0.clone(),
                    b_box: b0.clone(),
                });
                vec.push(Pair {
                    a_box: a0,
                    b_box: b1.clone(),
                });
                vec.push(Pair {
                    a_box: a1.clone(),
                    b_box: b0,
                });
                vec.push(Pair {
                    a_box: a1,
                    b_box: b1,
                });
            }
            (Some((a0, a1)), None) => {
                vec.push(Pair {
                    a_box: a0,
                    b_box: self.b_box.clone(),
                });
                vec.push(Pair {
                    a_box: a1,
                    b_box: self.b_box.clone(),
                });
            }
            (None, Some((b0, b1))) => {
                vec.push(Pair {
                    a_box: self.a_box.clone(),
                    b_box: b0,
                });
                vec.push(Pair {
                    a_box: self.a_box.clone(),
                    b_box: b1,
                });
            }
            (None, None) => vec.push(self.clone()),
        }
    }
}

impl XBox {
    #[inline]
    fn bisect(&self, space: &Space) -> Option<(XBox, XBox)> {
        let (col_0, col_1) = self.collider.bisect(space)?;
        let (pos_0, pos_1) = self.position.bisect();

        Some((
            XBox {
                position: pos_0,
                collider: col_0,
            },
            XBox {
                position: pos_1,
                collider: col_1,
            },
        ))
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

        let x0 = XBox {
            position: Default::default(),
            collider: a.into_collider(&space),
        };
        let x1 = XBox {
            position: Default::default(),
            collider: b.into_collider(&space),
        };
        let pair = Pair {
            a_box: x0,
            b_box: x1,
        };

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

        let x0 = XBox {
            position: Default::default(),
            collider: a.into_collider(&space),
        };
        let x1 = XBox {
            position: Default::default(),
            collider: b.into_collider(&space),
        };
        let pair = Pair {
            a_box: x0,
            b_box: x1,
        };

        assert_eq!(pair.overlap(&space), true);
    }
}
