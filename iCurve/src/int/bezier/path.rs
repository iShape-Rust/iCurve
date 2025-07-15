use crate::int::bezier::anchor::IntBezierAnchor;
use crate::int::bezier::spline::IntSpline;
use crate::int::math::point::IntPoint;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct IntBezierPath {
    pub anchors: Vec<IntBezierAnchor>,
    pub closed: bool,
}

impl IntBezierPath {

    #[inline]
    pub fn approximate_points(&self, min_cos: u32, min_len: u32) -> Vec<IntPoint> {
        let capacity = self.anchors.len() * 16;
        let mut points = Vec::with_capacity(capacity);
        for spline in self.splines() {
            points.append(&mut spline.approximate_points(min_cos, min_len));
        }

        points
    }

    #[inline]
    pub fn avg_length(&self, min_cos: u32, min_len: u32) -> u128 {
        let mut len = 0u128;
        for spline in self.splines() {
            len += spline.avg_length(min_cos, min_len);
        }

        len
    }

    #[inline]
    pub(crate) fn splines(&self) -> impl Iterator<Item = IntSpline> + '_ {
        IntSplineIterator::new(self)
    }
}

pub(crate) struct IntSplineIterator<'a> {
    path: &'a IntBezierPath,
    i: usize,
}

impl<'a> IntSplineIterator<'a> {
    #[inline]
    fn new(path: &'a IntBezierPath) -> Self {
        Self { i: 1, path }
    }
}

impl<'a> Iterator for IntSplineIterator<'a> {
    type Item = IntSpline;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.i > self.path.anchors.len() {
            return None;
        }

        if self.i == self.path.anchors.len() {
            self.i += 1;
            return if self.path.closed {
                let first = self.path.anchors.first().unwrap();
                let last = self.path.anchors.last().unwrap();
                Some(IntSpline::new(last, first))
            } else {
                None
            };
        }

        let a0 = &self.path.anchors[self.i - 1];
        let a1 = &self.path.anchors[self.i];

        self.i += 1;

        Some(IntSpline::new(a0, a1))
    }
}

#[cfg(test)]
mod tests {
    use crate::int::bezier::anchor::IntBezierAnchor;
    use crate::int::bezier::path::IntBezierPath;
    use crate::int::math::offset::IntOffset;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_00() {
        let path = IntBezierPath {
            anchors: vec![
                IntBezierAnchor {
                    point: IntPoint { x: -1000, y: 0 },
                    handle_in: Some(IntOffset { x: 0, y: -100 }),
                    handle_out: Some(IntOffset { x: 0, y: 100 }),
                },
                IntBezierAnchor {
                    point: IntPoint { x: 0, y: 1000 },
                    handle_in: Some(IntOffset { x: -100, y: 0 }),
                    handle_out: Some(IntOffset { x: 100, y: 0 }),
                },
                IntBezierAnchor {
                    point: IntPoint { x: 1000, y: 0 },
                    handle_in: Some(IntOffset { x: 0, y: 100 }),
                    handle_out: Some(IntOffset { x: 0, y: -100 }),
                },
                IntBezierAnchor {
                    point: IntPoint { x: 0, y: -1000 },
                    handle_in: Some(IntOffset { x: 100, y: 0 }),
                    handle_out: Some(IntOffset { x: -100, y: 0 }),
                },
            ],
            closed: true,
        };

        let points = path.approximate_points(800, 8);

        assert_eq!(points.len(), 20);
    }
}