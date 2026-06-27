use crate::kernel::float::cross::contact::ContactPoint;
use crate::kernel::float::curve::cubic::FloatCubicSegment;
use crate::kernel::float::curve::param::FloatSegmentParam;
use crate::kernel::float::curve::point_at::FloatPointAt;
use crate::kernel::float::curve::quad::FloatQuadSegment;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::i_float::float::point::FloatPoint;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CurveOverlap<T: FloatNumber> {
    pub point_0: ContactPoint<T>,
    pub point_1: ContactPoint<T>,
}

pub(crate) trait FindOverlap<T: FloatNumber> {
    fn find_overlap(&self, other: &Self, epsilon: T) -> Option<CurveOverlap<T>>;
}

pub(crate) trait OverlapSegment<T: FloatNumber>: FloatPointAt<T> {
    fn first_point(&self) -> FloatPoint<T>;
    fn last_point(&self) -> FloatPoint<T>;
    fn contains_point(&self, point: FloatPoint<T>, epsilon: T) -> Option<T>;
}

impl<T: FloatNumber> OverlapSegment<T> for FloatQuadSegment<T> {
    #[inline]
    fn first_point(&self) -> FloatPoint<T> {
        self.control_points[0]
    }

    #[inline]
    fn last_point(&self) -> FloatPoint<T> {
        self.control_points[2]
    }

    #[inline]
    fn contains_point(&self, point: FloatPoint<T>, epsilon: T) -> Option<T> {
        self.contains(point, epsilon)
    }
}

impl<T: FloatNumber> OverlapSegment<T> for FloatCubicSegment<T> {
    #[inline]
    fn first_point(&self) -> FloatPoint<T> {
        self.control_points[0]
    }

    #[inline]
    fn last_point(&self) -> FloatPoint<T> {
        self.control_points[3]
    }

    #[inline]
    fn contains_point(&self, point: FloatPoint<T>, epsilon: T) -> Option<T> {
        self.contains(point, epsilon)
    }
}

pub(crate) fn find_bezier_overlap<T: FloatNumber, S: OverlapSegment<T>>(
    segment_0: &S,
    segment_1: &S,
    epsilon: T,
    sample_count: usize,
) -> Option<CurveOverlap<T>> {
    let mut candidates = [None; 4];
    let mut count = 0;

    if let Some(t1) = segment_1.contains_point(segment_0.first_point(), epsilon) {
        push_unique_contact(
            &mut candidates,
            &mut count,
            ContactPoint {
                point: segment_0.first_point(),
                t0: FloatSegmentParam::Start,
                t1: FloatSegmentParam::from(t1),
            },
            epsilon,
        );
    }

    if let Some(t1) = segment_1.contains_point(segment_0.last_point(), epsilon) {
        push_unique_contact(
            &mut candidates,
            &mut count,
            ContactPoint {
                point: segment_0.last_point(),
                t0: FloatSegmentParam::End,
                t1: FloatSegmentParam::from(t1),
            },
            epsilon,
        );
    }

    if let Some(t0) = segment_0.contains_point(segment_1.first_point(), epsilon) {
        push_unique_contact(
            &mut candidates,
            &mut count,
            ContactPoint {
                point: segment_1.first_point(),
                t0: FloatSegmentParam::from(t0),
                t1: FloatSegmentParam::Start,
            },
            epsilon,
        );
    }

    if let Some(t0) = segment_0.contains_point(segment_1.last_point(), epsilon) {
        push_unique_contact(
            &mut candidates,
            &mut count,
            ContactPoint {
                point: segment_1.last_point(),
                t0: FloatSegmentParam::from(t0),
                t1: FloatSegmentParam::End,
            },
            epsilon,
        );
    }

    let (point_0, point_1) = farthest_contact_pair(&candidates, count, epsilon)?;
    if !is_valid_overlap(segment_0, segment_1, point_0, point_1, epsilon, sample_count) {
        return None;
    }

    Some(CurveOverlap { point_0, point_1 })
}

fn push_unique_contact<T: FloatNumber>(
    candidates: &mut [Option<ContactPoint<T>>; 4],
    count: &mut usize,
    contact: ContactPoint<T>,
    epsilon: T,
) {
    let eps_sqr = epsilon * epsilon;
    for candidate in candidates.iter().take(*count).flatten() {
        if (candidate.point - contact.point).sqr_length() <= eps_sqr
            && candidate.t0.compare_with_epsilon(contact.t0, epsilon)
            && candidate.t1.compare_with_epsilon(contact.t1, epsilon)
        {
            return;
        }
    }

    if *count < candidates.len() {
        candidates[*count] = Some(contact);
        *count += 1;
    }
}

fn farthest_contact_pair<T: FloatNumber>(
    candidates: &[Option<ContactPoint<T>>; 4],
    count: usize,
    epsilon: T,
) -> Option<(ContactPoint<T>, ContactPoint<T>)> {
    let mut best = None;
    let mut best_sqr_distance = T::ZERO;

    for i in 0..count {
        let Some(a) = candidates[i] else {
            continue;
        };

        for b in candidates.iter().take(count).skip(i + 1).flatten() {
            let sqr_distance = (a.point - b.point).sqr_length();
            if sqr_distance > best_sqr_distance {
                best_sqr_distance = sqr_distance;
                best = Some((a, *b));
            }
        }
    }

    if best_sqr_distance <= epsilon * epsilon {
        return None;
    }

    best
}

fn is_valid_overlap<T: FloatNumber, S: OverlapSegment<T>>(
    segment_0: &S,
    segment_1: &S,
    point_0: ContactPoint<T>,
    point_1: ContactPoint<T>,
    epsilon: T,
    sample_count: usize,
) -> bool {
    let eps_sqr = epsilon * epsilon;
    let t00 = point_0.t0.value();
    let t01 = point_1.t0.value();
    let t10 = point_0.t1.value();
    let t11 = point_1.t1.value();

    for i in 1..=sample_count {
        let ratio = T::from_float(i as f64 / (sample_count + 1) as f64);
        let t0 = t00 + (t01 - t00) * ratio;
        let t1 = t10 + (t11 - t10) * ratio;
        let p0 = segment_0.point_at(FloatSegmentParam::from(t0));
        let p1 = segment_1.point_at(FloatSegmentParam::from(t1));

        if (p0 - p1).sqr_length() > eps_sqr {
            return false;
        }
    }

    true
}
