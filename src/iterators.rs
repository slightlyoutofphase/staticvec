use crate::utils::distance_between;
use crate::StaticVec;
use core::fmt::{self, Debug, Formatter};
use core::intrinsics;
use core::iter::{FusedIterator, TrustedLen};
use core::marker::{PhantomData, Send, Sync};
use core::ptr;
use core::slice;

#[cfg(feature = "std")]
use alloc::string::String;

#[cfg(feature = "std")]
use alloc::format;

/// Similar to [`Iter`](core::slice::Iter), but specifically implemented with StaticVecs in mind.
pub struct StaticVecIterConst<'a, T: 'a, const N: usize> {
  pub(crate) start: *const T,
  pub(crate) end: *const T,
  pub(crate) marker: PhantomData<&'a T>,
}

/// Similar to [`IterMut`](core::slice::IterMut), but specifically implemented with StaticVecs in
/// mind.
pub struct StaticVecIterMut<'a, T: 'a, const N: usize> {
  pub(crate) start: *mut T,
  pub(crate) end: *mut T,
  pub(crate) marker: PhantomData<&'a mut T>,
}

/// A "consuming" iterator that reads each element out of
/// a source StaticVec by value.
pub struct StaticVecIntoIter<T, const N: usize> {
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) data: StaticVec<T, N>,
}

impl<'a, T: 'a, const N: usize> StaticVecIterConst<'a, T, N> {
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    // Safety: `start` and `end` are never null.
    unsafe {
      format!(
        "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
        self.start.as_ref(),
        self.end.offset(-1).as_ref()
      )
    }
  }

  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` pointers.
  #[inline(always)]
  pub fn as_slice(&self) -> &'a [T] {
    // Safety: `start` is never null. This function will "at worst" return an empty slice.
    unsafe { slice::from_raw_parts(self.start, self.len()) }
  }
}

impl<'a, T: 'a, const N: usize> Iterator for StaticVecIterConst<'a, T, N> {
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

impl<'a, T: 'a, const N: usize> DoubleEndedIterator for StaticVecIterConst<'a, T, N> {
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

impl<'a, T: 'a, const N: usize> ExactSizeIterator for StaticVecIterConst<'a, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a, const N: usize> FusedIterator for StaticVecIterConst<'a, T, N> {}
unsafe impl<'a, T: 'a, const N: usize> TrustedLen for StaticVecIterConst<'a, T, N> {}
unsafe impl<'a, T: 'a + Sync, const N: usize> Sync for StaticVecIterConst<'a, T, N> {}
unsafe impl<'a, T: 'a + Sync, const N: usize> Send for StaticVecIterConst<'a, T, N> {}

impl<'a, T: 'a, const N: usize> Clone for StaticVecIterConst<'a, T, N> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      start: self.start,
      end: self.end,
      marker: self.marker,
    }
  }
}

impl<'a, T: 'a + Debug, const N: usize> Debug for StaticVecIterConst<'a, T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_list().entries(self.as_slice()).finish()
  }
}

impl<'a, T: 'a, const N: usize> StaticVecIterMut<'a, T, N> {
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    // Safety: `start` and `end` are never null.
    unsafe {
      format!(
        "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
        self.start.as_ref(),
        self.end.offset(-1).as_ref()
      )
    }
  }

  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` pointers. Though this is a mutable iterator, the slice cannot be mutable
  /// as it would lead to aliasing issues.
  #[inline(always)]
  pub fn as_slice(&self) -> &'a [T] {
    // Safety: `start` is never null. This function will "at worst" return an empty slice.
    unsafe { slice::from_raw_parts(self.start, self.len()) }
  }
}

impl<'a, T: 'a, const N: usize> Iterator for StaticVecIterMut<'a, T, N> {
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

impl<'a, T: 'a, const N: usize> DoubleEndedIterator for StaticVecIterMut<'a, T, N> {
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

impl<'a, T: 'a, const N: usize> ExactSizeIterator for StaticVecIterMut<'a, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a, const N: usize> FusedIterator for StaticVecIterMut<'a, T, N> {}
unsafe impl<'a, T: 'a, const N: usize> TrustedLen for StaticVecIterMut<'a, T, N> {}
unsafe impl<'a, T: 'a + Sync, const N: usize> Sync for StaticVecIterMut<'a, T, N> {}
unsafe impl<'a, T: 'a + Sync, const N: usize> Send for StaticVecIterMut<'a, T, N> {}

impl<'a, T: 'a + Debug, const N: usize> Debug for StaticVecIterMut<'a, T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_list().entries(self.as_slice()).finish()
  }
}

impl<T, const N: usize> StaticVecIntoIter<T, N> {
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    // Safety: `start` and `end` are never out of bounds.
    unsafe {
      format!(
        "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
        self.data.data.get_unchecked(self.start).get_ref(),
        self.data.data.get_unchecked(self.end - 1).get_ref()
      )
    }
  }

  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` indices.
  #[inline(always)]
  pub fn as_slice(&self) -> &[T] {
    // Safety: `start` is never null. This function will "at worst" return an empty slice.
    unsafe { slice::from_raw_parts(self.data.ptr_at_unchecked(self.start), self.len()) }
  }
}

impl<T, const N: usize> Iterator for StaticVecIntoIter<T, N> {
  type Item = T;
  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    match self.end - self.start {
      0 => None,
      _ => {
        let res = Some(unsafe { self.data.ptr_at_unchecked(self.start).read() });
        self.start += 1;
        res
      }
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.end - self.start;
    (len, Some(len))
  }
}

impl<T, const N: usize> DoubleEndedIterator for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<Self::Item> {
    match self.end - self.start {
      0 => None,
      _ => {
        self.end -= 1;
        Some(unsafe { self.data.ptr_at_unchecked(self.end).read() })
      }
    }
  }
}

impl<T, const N: usize> ExactSizeIterator for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.end - self.start
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.end - self.start == 0
  }
}

impl<T, const N: usize> FusedIterator for StaticVecIntoIter<T, N> {}
unsafe impl<T, const N: usize> TrustedLen for StaticVecIntoIter<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for StaticVecIntoIter<T, N> {}
unsafe impl<T: Sync, const N: usize> Send for StaticVecIntoIter<T, N> {}

impl<T: Debug, const N: usize> Debug for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_list().entries(self.as_slice()).finish()
  }
}

impl<T, const N: usize> Drop for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn drop(&mut self) {
    let item_count = self.len();
    match item_count {
      0 => (),
      _ => unsafe {
        ptr::drop_in_place(slice::from_raw_parts_mut(
          self.data.mut_ptr_at_unchecked(self.start),
          item_count,
        ))
      },
    }
  }
}
