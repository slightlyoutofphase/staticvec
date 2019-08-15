use crate::utils::distance_between;
use core::iter::{FusedIterator, TrustedLen};
use core::marker::PhantomData;

///Similar to [Iter](core::slice::IterMut), but specifically implemented with StaticVecs in mind.
pub struct StaticVecIterConst<'a, T: 'a> {
  pub(crate) start: *const T,
  pub(crate) end: *const T,
  pub(crate) marker: PhantomData<&'a T>,
}

///Similar to [IterMut](core::slice::IterMut), but specifically implemented with StaticVecs in mind.
pub struct StaticVecIterMut<'a, T: 'a> {
  pub(crate) start: *mut T,
  pub(crate) end: *mut T,
  pub(crate) marker: PhantomData<&'a mut T>,
}

impl<'a, T: 'a> Iterator for StaticVecIterConst<'a, T> {
  type Item = &'a T;
  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    if self.start < self.end {
      unsafe {
        let res = Some(&*self.start);
        self.start = self.start.offset(1);
        res
      }
    } else {
      None
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = distance_between(self.end, self.start);
    (len, Some(len))
  }
}

impl<'a, T: 'a> DoubleEndedIterator for StaticVecIterConst<'a, T> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.end > self.start {
      unsafe {
        self.end = self.end.offset(-1);
        Some(&*self.end)
      }
    } else {
      None
    }
  }
}

impl<'a, T: 'a> ExactSizeIterator for StaticVecIterConst<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a> FusedIterator for StaticVecIterConst<'a, T> {}
impl<'a, T: 'a> TrustedLen for StaticVecIterConst<'a, T> {}

impl<'a, T: 'a> Iterator for StaticVecIterMut<'a, T> {
  type Item = &'a mut T;
  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    if self.start < self.end {
      unsafe {
        let res = Some(&mut *self.start);
        self.start = self.start.offset(1);
        res
      }
    } else {
      None
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = distance_between(self.end, self.start);
    (len, Some(len))
  }
}

impl<'a, T: 'a> DoubleEndedIterator for StaticVecIterMut<'a, T> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.end > self.start {
      unsafe {
        self.end = self.end.offset(-1);
        Some(&mut *self.end)
      }
    } else {
      None
    }
  }
}

impl<'a, T: 'a> ExactSizeIterator for StaticVecIterMut<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a> FusedIterator for StaticVecIterMut<'a, T> {}
impl<'a, T: 'a> TrustedLen for StaticVecIterMut<'a, T> {}
