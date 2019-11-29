use crate::utils::distance_between;
use core::fmt::{self, Debug, Formatter};
use core::intrinsics;
use core::iter::{FusedIterator, TrustedLen};
use core::marker::{PhantomData, Send, Sync};
use core::slice;

#[cfg(feature = "std")]
use alloc::string::String;

#[cfg(feature = "std")]
use alloc::format;

/// Similar to [Iter](core::slice::Iter), but specifically implemented with StaticVecs in mind.
pub struct StaticVecIterConst<'a, T: 'a> {
  pub(crate) start: *const T,
  pub(crate) end: *const T,
  pub(crate) marker: PhantomData<&'a T>,
}

/// Similar to [IterMut](core::slice::IterMut), but specifically implemented with StaticVecs in
/// mind.
pub struct StaticVecIterMut<'a, T: 'a> {
  pub(crate) start: *mut T,
  pub(crate) end: *mut T,
  pub(crate) marker: PhantomData<&'a mut T>,
}

impl<'a, T: 'a> StaticVecIterConst<'a, T> {
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    // Safety: `start` and `end` are never null.
    unsafe {
      format!(
        "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
        *self.start,
        *self.end.offset(-1)
      )
    }
  }

  #[inline(always)]
  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` pointers.
  pub fn as_slice(&self) -> &'a [T] {
    // Safety: `start` is never null. This function will "at worst" return an empty slice.
    unsafe { slice::from_raw_parts(self.start, self.len()) }
  }
}

impl<'a, T: 'a> Iterator for StaticVecIterConst<'a, T> {
  type Item = &'a T;
  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    match distance_between(self.end, self.start) {
      0 => None,
      _ => unsafe {
        let res = Some(&*self.start);
        self.start = match intrinsics::size_of::<T>() {
          0 => (self.start as usize + 1) as *const _,
          _ => self.start.offset(1),
        };
        res
      },
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
    match distance_between(self.end, self.start) {
      0 => None,
      _ => unsafe {
        self.end = match intrinsics::size_of::<T>() {
          0 => (self.end as usize - 1) as *const _,
          _ => self.end.offset(-1),
        };
        Some(&*self.end)
      },
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
unsafe impl<'a, T: 'a> TrustedLen for StaticVecIterConst<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Sync for StaticVecIterConst<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Send for StaticVecIterConst<'a, T> {}

impl<'a, T: 'a> Clone for StaticVecIterConst<'a, T> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      start: self.start,
      end: self.end,
      marker: self.marker,
    }
  }
}

impl<'a, T: 'a + Debug> Debug for StaticVecIterConst<'a, T> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Debug::fmt(self.as_slice(), f)
  }
}

impl<'a, T: 'a> StaticVecIterMut<'a, T> {
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    // Safety: `start` and `end` are never null.
    unsafe {
      format!(
        "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
        *self.start,
        *self.end.offset(-1)
      )
    }
  }

  #[inline(always)]
  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` pointers. Though this is a mutable iterator, the slice cannot be mutable
  /// as it would lead to aliasing issues.
  pub fn as_slice(&self) -> &'a [T] {
    // Safety: `start` is never null. This function will "at worst" return an empty slice.
    unsafe { slice::from_raw_parts(self.start, self.len()) }
  }
}

impl<'a, T: 'a> Iterator for StaticVecIterMut<'a, T> {
  type Item = &'a mut T;
  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    match distance_between(self.end, self.start) {
      0 => None,
      _ => unsafe {
        let res = Some(&mut *self.start);
        self.start = match intrinsics::size_of::<T>() {
          0 => (self.start as usize + 1) as *mut _,
          _ => self.start.offset(1),
        };
        res
      },
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
    match distance_between(self.end, self.start) {
      0 => None,
      _ => unsafe {
        self.end = match intrinsics::size_of::<T>() {
          0 => (self.end as usize - 1) as *mut _,
          _ => self.end.offset(-1),
        };
        Some(&mut *self.end)
      },
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
unsafe impl<'a, T: 'a> TrustedLen for StaticVecIterMut<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Sync for StaticVecIterMut<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Send for StaticVecIterMut<'a, T> {}

impl<'a, T: 'a + Debug> Debug for StaticVecIterMut<'a, T> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Debug::fmt(self.as_slice(), f)
  }
}
