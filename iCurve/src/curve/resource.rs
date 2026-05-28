use crate::curve::contour::CurveContour;
use crate::curve::shape::CurveShape;
use alloc::vec::Vec;
use i_overlay::i_float::float::compatible::FloatPointCompatible;

pub trait CurveResource<P>
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>: Iterator<Item = &'a CurveContour<P>>
    where
        P: 'a,
        Self: 'a;

    fn iter_contours(&self) -> Self::ContourIter<'_>;
}

impl<P, R> CurveResource<P> for &R
where
    P: FloatPointCompatible,
    R: CurveResource<P> + ?Sized,
{
    type ContourIter<'a>
        = R::ContourIter<'a>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        (**self).iter_contours()
    }
}

pub struct SingleContourResourceIterator<'a, P: FloatPointCompatible> {
    contour: Option<&'a CurveContour<P>>,
}

impl<'a, P: FloatPointCompatible> SingleContourResourceIterator<'a, P> {
    #[inline]
    fn with_contour(contour: &'a CurveContour<P>) -> Self {
        Self {
            contour: Some(contour),
        }
    }
}

impl<'a, P: FloatPointCompatible> Iterator for SingleContourResourceIterator<'a, P> {
    type Item = &'a CurveContour<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.contour.take()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.contour.is_some() as usize
    }
}

pub struct ContoursResourceIterator<'a, P: FloatPointCompatible> {
    contours: &'a [CurveContour<P>],
    index: usize,
}

impl<'a, P: FloatPointCompatible> ContoursResourceIterator<'a, P> {
    #[inline]
    fn with_slice(contours: &'a [CurveContour<P>]) -> Self {
        Self { contours, index: 0 }
    }
}

impl<'a, P: FloatPointCompatible> Iterator for ContoursResourceIterator<'a, P> {
    type Item = &'a CurveContour<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.contours.len() {
            return None;
        }
        let index = self.index;
        self.index += 1;
        Some(&self.contours[index])
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.contours.len().saturating_sub(self.index)
    }
}

pub struct ShapesResourceIterator<'a, P: FloatPointCompatible> {
    shapes: &'a [CurveShape<P>],
    shape_index: usize,
    contour_index: usize,
}

impl<'a, P: FloatPointCompatible> ShapesResourceIterator<'a, P> {
    #[inline]
    fn with_slice(shapes: &'a [CurveShape<P>]) -> Self {
        Self {
            shapes,
            shape_index: 0,
            contour_index: 0,
        }
    }
}

impl<'a, P: FloatPointCompatible> Iterator for ShapesResourceIterator<'a, P> {
    type Item = &'a CurveContour<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.shape_index < self.shapes.len() {
            let shape = &self.shapes[self.shape_index];
            if self.contour_index < shape.contours.len() {
                let contour = &shape.contours[self.contour_index];
                self.contour_index += 1;
                return Some(contour);
            }
            self.shape_index += 1;
            self.contour_index = 0;
        }

        None
    }
}

impl<P> CurveResource<P> for CurveContour<P>
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = SingleContourResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        SingleContourResourceIterator::with_contour(self)
    }
}

impl<P> CurveResource<P> for [CurveContour<P>]
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = ContoursResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        ContoursResourceIterator::with_slice(self)
    }
}

impl<P, const N: usize> CurveResource<P> for [CurveContour<P>; N]
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = ContoursResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        ContoursResourceIterator::with_slice(self)
    }
}

impl<P> CurveResource<P> for Vec<CurveContour<P>>
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = ContoursResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        ContoursResourceIterator::with_slice(self.as_slice())
    }
}

impl<P> CurveResource<P> for CurveShape<P>
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = ContoursResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        ContoursResourceIterator::with_slice(self.contours.as_slice())
    }
}

impl<P> CurveResource<P> for [CurveShape<P>]
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = ShapesResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        ShapesResourceIterator::with_slice(self)
    }
}

impl<P, const N: usize> CurveResource<P> for [CurveShape<P>; N]
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = ShapesResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        ShapesResourceIterator::with_slice(self)
    }
}

impl<P> CurveResource<P> for Vec<CurveShape<P>>
where
    P: FloatPointCompatible,
{
    type ContourIter<'a>
        = ShapesResourceIterator<'a, P>
    where
        P: 'a,
        Self: 'a;

    #[inline]
    fn iter_contours(&self) -> Self::ContourIter<'_> {
        ShapesResourceIterator::with_slice(self.as_slice())
    }
}
