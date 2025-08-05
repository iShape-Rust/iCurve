use core::ptr;

#[derive(Clone)]
pub struct FourVec<T: Copy + Default> {
    pub(crate) buffer: [T; 4],
    pub(crate) len: usize,
}

impl<T: Copy + Default> FourVec<T> {

    #[inline]
    pub fn slice(&self) -> &[T] {
        &self.buffer[0..self.len]
    }
    
    #[inline]
    pub(crate) fn new() -> Self {
        Self { buffer: [T::default(); 4], len: 0 }
    }

    #[inline]
    pub(crate) fn with_slice(src: &[T]) -> Self {
        debug_assert!(src.len() <= 4);
        let mut buf = [T::default(); 4];
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), buf.as_mut_ptr(), src.len());
        }
        Self { buffer: buf, len: src.len() }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool { self.len == 0 }

    #[inline]
    pub(crate) fn push(&mut self, value: T) {
        debug_assert!(self.len < 4);
        self.buffer[self.len] = value;
        self.len += 1;
    }

    #[inline]
    pub(crate) fn remove(&mut self, idx: usize) {
        debug_assert!(idx < self.len);
        let last = self.len - 1;
        self.len -= 1;
        self.buffer[idx] = self.buffer[last];
    }

    #[inline]
    pub(crate) fn extract(&mut self, idx: usize) -> T {
        let val = self.buffer[idx];
        self.remove(idx);
        val
    }
}