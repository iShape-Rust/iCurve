use crate::int::bezier::position::LineDivider;
use crate::int::bezier::spline::{IntBezierSplineApi, SplitPosition};
use crate::int::math::normalize::VectorNormalization16;
use crate::int::math::point::IntPoint;
use crate::int::math::rect::IntRect;

#[derive(Debug, Clone)]
pub struct IntCubeSpline {
    pub anchors: [IntPoint; 4],
}
impl IntBezierSplineApi for IntCubeSpline {
    #[inline]
    fn start(&self) -> IntPoint {
        self.anchors[0]
    }
    #[inline]
    fn start_dir(&self) -> IntPoint {
        (self.anchors[1] - self.anchors[0]).normalized_16bit()
    }
    #[inline]
    fn end_dir(&self) -> IntPoint {
        (self.anchors[3] - self.anchors[2]).normalized_16bit()
    }
    #[inline]
    fn end(&self) -> IntPoint {
        self.anchors[3]
    }

    #[inline]
    fn point_at(&self, position: &SplitPosition) -> IntPoint {
        let [a, ma, mb, b] = self.anchors;

        let m0 = LineDivider::new(a, ma).point_at(position);
        let m1 = LineDivider::new(ma, mb).point_at(position);
        let m2 = LineDivider::new(mb, b).point_at(position);

        let mm0 = LineDivider::new(m0, m1).point_at(position);
        let mm1 = LineDivider::new(m1, m2).point_at(position);

        LineDivider::new(mm0, mm1).point_at(position)
    }

    #[inline]
    fn bisect(&self) -> (Self, Self) {
        let [a, ma, mb, b] = self.anchors;

        let m0 = a.mid(&ma);
        let m1 = ma.mid(&mb);
        let m2 = mb.mid(&b);

        let mm0 = m0.mid(&m1);
        let mm1 = m1.mid(&m2);

        let mmm = mm0.mid(&mm1);

        let anchors_0 = [a, m0, mm0, mmm];
        let anchors_1 = [mmm, mm1, m2, b];

        (Self { anchors: anchors_0 }, Self { anchors: anchors_1 })
    }

    #[inline]
    fn split(&self, position: &SplitPosition) -> (Self, Self) {
        let [a, ma, mb, b] = self.anchors;

        let m0 = LineDivider::new(a, ma).point_at(position);
        let m1 = LineDivider::new(ma, mb).point_at(position);
        let m2 = LineDivider::new(mb, b).point_at(position);

        let mm0 = LineDivider::new(m0, m1).point_at(position);
        let mm1 = LineDivider::new(m1, m2).point_at(position);

        let mmm = LineDivider::new(mm0, mm1).point_at(position);

        let anchors_0 = [a, m0, mm0, mmm];
        let anchors_1 = [mmm, mm1, m2, b];

        (Self { anchors: anchors_0 }, Self { anchors: anchors_1 })
    }

    #[inline]
    fn boundary(&self) -> IntRect {
        IntRect::with_points(&self.anchors)
    }

    #[inline]
    fn anchors(&self) -> &[IntPoint] {
        &self.anchors
    }
}

impl IntCubeSpline {
    #[inline]
    pub(crate) fn is_flat(&self, power: u32, generation: u32) -> bool {
        // power max(boundary.size) < 2^power
        // generation 0..64
        if generation >= 20 {
            return true;
        }

        if power <= 3 {
            // spline is already too small
            return true;
        } else if power < 30 {
            return self.is_flat_i32(generation);
        }

        false
    }

    #[inline]
    fn is_flat_i32(&self, generation: u32) -> bool {
        let [a, ma, mb, b] = self.anchors;

        let v = b - a;
        let va = ma - a;
        let vb = b - mb;

        // we wish to compare Sa and Sb and S
        // Sa = cross_a = sinA * |va| * |v|
        // Sb = cross_b = sinB * |vb| * |v|
        // S = max_h * |v|
        // max_h = 1 << generation
        // Sa < S and Sb < S

        let sa = v.cross_product(&va);
        let sb = vb.cross_product(&v);

        if sa <= 0 || sb <= 0 {
            return false
        }

        let v_log2len = v.sqr_len().ilog2() / 2;
        let s = 1i64 << (v_log2len + generation);

        sa < s && sb < s
    }
}

#[cfg(test)]
mod tests {
    use crate::int::bezier::spline_cube::IntCubeSpline;
    use crate::int::math::point::IntPoint;

    #[test]
    fn test_0() {
        let spline = IntCubeSpline {
            anchors: [
                IntPoint::new(0, 0),
                IntPoint::new(100, 100),
                IntPoint::new(300, 100),
                IntPoint::new(400, 0),
            ],
        };

        assert_eq!(false, spline.is_flat(8, 0));
    }

    #[test]
    fn test_1() {
        let spline = IntCubeSpline {
            anchors: [
                IntPoint::new(0, 0),
                IntPoint::new(100, 1),
                IntPoint::new(300, 1),
                IntPoint::new(400, 0),
            ],
        };

        assert_eq!(false, spline.is_flat(8, 0));
        assert_eq!(true, spline.is_flat(8, 1));
    }
}
