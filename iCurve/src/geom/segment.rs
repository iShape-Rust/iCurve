use i_overlay::i_float::int::number::int::IntNumber;
use i_overlay::i_float::int::number::wide_int::WideIntNumber;
use i_overlay::i_shape::int::IntPoint;

pub(crate) struct Segment<I: IntNumber> {
    a: IntPoint<I>,
    b: IntPoint<I>,
}

pub(crate) enum SegmentProximity {
    Intersect,
    Close,
    Separate,
}
impl<I: IntNumber> Segment<I> {

    pub(crate) fn proximity_to(&self, other: &Segment<I>, distance: I) -> SegmentProximity {
        let a = self.a;
        let b = self.b;
        let c = other.a;
        let d = other.b;

        let ab = b - a;
        let cd = d - c;

        // c - ab, d - ab

        let ac = c - a;
        let ad = d - a;

        // a - cd, b - cd
        let ca = a - c;
        let cb = b - c;


        let ab_x_ac = ab.cross_product(ac);
        let ab_x_ad = ab.cross_product(ad);
        let cd_x_ca = cd.cross_product(ca);
        let cd_x_cb = cd.cross_product(cb);

        let ab_sign_test = ab_x_ac < I::Wide::ZERO && I::Wide::ZERO > ab_x_ad || ab_x_ac > I::Wide::ZERO && I::Wide::ZERO < ab_x_ad;
        let cd_sign_test = cd_x_ca < I::Wide::ZERO && I::Wide::ZERO > cd_x_cb || cd_x_ca > I::Wide::ZERO && I::Wide::ZERO < cd_x_cb;

        if ab_sign_test && cd_sign_test {
            return SegmentProximity::Intersect;
        }

        SegmentProximity::Separate
    }
}
