use core::fmt::{self, Debug, Formatter};
use core::intrinsics::assume;
use core::iter::{FusedIterator, TrustedLen, TrustedRandomAccessNoCoerce};
use core::marker::{PhantomData, Send, Sync};
use core::mem::{replace, size_of, MaybeUninit};
use core::ptr;
use core::slice::{from_raw_parts, from_raw_parts_mut};

use crate::utils::{distance_between, zst_ptr_add, zst_ptr_add_mut};
use crate::StaticVec;

#[cfg(feature = "std")]
use alloc::string::String;

#[cfg(feature = "std")]
use alloc::format;

// Note that all of the iterators in this file having a constant generic `N` parameter is not really
// useful currently, but done in a forward-thinking sense for such a time when it *is* useful in
// order to avoid unsolvable backwards-compatibility issues at that point.

/// Similar to [`Iter`](core::slice::Iter), but specifically implemented with [`StaticVec`]s in
/// mind.
pub struct StaticVecIterConst<'a, T: 'a, const N: usize> {
  pub(crate) start: *const T,
  pub(crate) end: *const T,
  pub(crate) marker: PhantomData<&'a T>,
}

/// Similar to [`IterMut`](core::slice::IterMut), but specifically implemented with [`StaticVec`]s
/// in mind.
pub struct StaticVecIterMut<'a, T: 'a, const N: usize> {
  pub(crate) start: *mut T,
  pub(crate) end: *mut T,
  pub(crate) marker: PhantomData<&'a mut T>,
}

/// A "consuming" iterator that reads each element out of a source [`StaticVec`] by value.
pub struct StaticVecIntoIter<T, const N: usize> {
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) data: MaybeUninit<[T; N]>,
}

/// A "draining" iterator, analogous to [`vec::Drain`](alloc::vec::Drain).
/// Instances of [`StaticVecDrain`](crate::iterators::StaticVecDrain) are created
/// by the [`drain_iter`](crate::StaticVec::drain_iter) method on [`StaticVec`](crate::StaticVec),
/// as while the [`drain`](crate::StaticVec::drain) method does have a similar purpose, it works by
/// immediately returning a new [`StaticVec`](crate::StaticVec) as opposed to an iterator.
pub struct StaticVecDrain<'a, T: 'a, const N: usize> {
  pub(crate) start: usize,
  pub(crate) length: usize,
  pub(crate) iter: StaticVecIterConst<'a, T, N>,
  pub(crate) vec: *mut StaticVec<T, N>,
}

/// A "splicing" iterator, analogous to [`vec::Splice`](alloc::vec::Splice).
/// Instances of [`StaticVecSplice`](crate::iterators::StaticVecSplice) are created
/// by the [`splice`](crate::StaticVec::splice) method on [`StaticVec`](crate::StaticVec).
pub struct StaticVecSplice<T, I: Iterator<Item = T>, const N: usize> {
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) replace_with: I,
  pub(crate) vec: *mut StaticVec<T, N>,
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
    match self.len() {
      0 => String::from("Empty iterator!"),
      _ => unsafe {
        // Safety: `start` and `end` are never null.
        format!(
          "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
          &*self.start,
          &*self.end.offset(-1)
        )
      },
    }
  }

  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` pointers.
  #[inline(always)]
  pub const fn as_slice(&self) -> &'a [T] {
    // Safety: `start` is never null. This function will "at worst" return an empty slice.
    unsafe { from_raw_parts(self.start, distance_between(self.end, self.start)) }
  }
}

// Note that the `const` iterator implementations below are `const` moreso just as groundwork for a
// future where `for` loops are allowed in `const` contexts, and do not necessarily benefit in any
// particularly useful way from being `const` impls quite yet.

impl<'a, T: 'a, const N: usize> Iterator for StaticVecIterConst<'a, T, N> {
  type Item = &'a T;

  #[inline(always)]
  fn next(&mut self) -> Option<&'a T> {
    unsafe {
      // Safety: `self.start` and `self.end` are never null if `T` is not a ZST,
      // and the possibility that `self.end` specifically is null if `T` *is* a ZST
      // is accounted for.
      assume(!self.start.is_null());
      if size_of::<T>() != 0 {
        assume(!self.end.is_null());
      }
      match distance_between(self.end, self.start) {
        0 => None,
        _ => {
          let res = Some(&*self.start);
          match size_of::<T>() {
            0 => self.end = (self.end as *const u8).wrapping_offset(-1) as *const T;
            _ => self.start = self.start.offset(1),
          }
          res
        }
      }
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = distance_between(self.end, self.start);
    (len, Some(len))
  }

  #[inline(always)]
  fn count(self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn nth(&mut self, n: usize) -> Option<&'a T> {
    if n >= distance_between(self.end, self.start) {
      None
    } else {
      unsafe {
        match size_of::<T>() {
          0 => {
            self.end = (self.end as *const u8).wrapping_offset(-(n as isize)) as *const T;
            self.start = (self.start as usize + 1) as *const T;
            Some(&*self.start)
          }
          _ => {
            let res = self.start.add(n);
            self.start = res.offset(1);
            Some(&*res)
          }
        }
      }
    }
  }

  #[inline(always)]
  fn last(mut self) -> Option<&'a T> {
    self.next_back()
  }

  #[inline(always)]
  unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> &'a T {
    // Safety: see the comments for the implementation of this function in
    // 'core/slice/iter/macros.rs`, as they're equally applicable to us here.
    &*self.start.add(idx)
  }
}

impl<'a, T: 'a, const N: usize> DoubleEndedIterator for StaticVecIterConst<'a, T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<&'a T> {
    unsafe {
      // Safety: `self.start` and `self.end` are never null if `T` is not a ZST,
      // and the possibility that `self.end` specifically is null if `T` *is* a ZST
      // is accounted for.
      assume(!self.start.is_null());
      if size_of::<T>() != 0 {
        assume(!self.end.is_null());
      }
      match distance_between(self.end, self.start) {
        0 => None,
        _ => {
          self.end = match size_of::<T>() {
            0 => (self.end as usize - 1) as *const T,
            _ => self.end.offset(-1),
          };
          Some(&*self.end)
        }
      }
    }
  }

  #[inline(always)]
  fn nth_back(&mut self, n: usize) -> Option<&'a T> {
    if n >= distance_between(self.end, self.start) {
      None
    } else {
      unsafe {
        self.end = match size_of::<T>() {
          0 => (self.end as *const u8).wrapping_offset(-((n as isize) + 1)) as *const T,
          _ => self.end.offset(-((n as isize) + 1)),
        };
        Some(&*self.end)
      }
    }
  }
}

impl<'a, T: 'a, const N: usize> const ExactSizeIterator for StaticVecIterConst<'a, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a, const N: usize> const FusedIterator for StaticVecIterConst<'a, T, N> {}
unsafe impl<'a, T: 'a, const N: usize> const TrustedLen for StaticVecIterConst<'a, T, N> {}
// We hide this one just in case it gets removed from `std` later, so no one relies on us relying on
// it.
#[doc(hidden)]
unsafe impl<'a, T: 'a, const N: usize> const TrustedRandomAccessNoCoerce
  for StaticVecIterConst<'a, T, N>
{
  const MAY_HAVE_SIDE_EFFECT: bool = false;
  fn size(&self) -> usize {
    distance_between(self.end, self.start)
  }
}
unsafe impl<'a, T: 'a + Sync, const N: usize> const Sync for StaticVecIterConst<'a, T, N> {}
// `StaticVecIterConst` works in terms of `&T`, which is sendable if / when `T` is `Sync`.
unsafe impl<'a, T: 'a + Sync, const N: usize> const Send for StaticVecIterConst<'a, T, N> {}

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
    f.debug_tuple("StaticVecIterConst")
      .field(&self.as_slice())
      .finish()
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
    match self.len() {
      0 => String::from("Empty iterator!"),
      _ => unsafe {
        // Safety: `start` and `end` are never null.
        format!(
          "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
          &*self.start,
          &*self.end.offset(-1)
        )
      },
    }
  }

  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` pointers. Though this is a mutable iterator, the slice cannot be mutable
  /// as it would lead to aliasing issues.
  #[inline(always)]
  pub const fn as_slice(&self) -> &[T] {
    // Safety: `start` is never null. This function will "at worst" return an empty slice.
    unsafe { from_raw_parts(self.start, distance_between(self.end, self.start)) }
  }
}

impl<'a, T: 'a, const N: usize> Iterator for StaticVecIterMut<'a, T, N> {
  type Item = &'a mut T;

  #[inline(always)]
  fn next(&mut self) -> Option<&'a mut T> {
    unsafe {
      // Safety: `self.start` and `self.end` are never null if `T` is not a ZST,
      // and the possibility that `self.end` specifically is null if `T` *is* a ZST
      // is accounted for.
      assume(!self.start.is_null());
      if size_of::<T>() != 0 {
        assume(!self.end.is_null());
      }
      match distance_between(self.end, self.start) {
        0 => None,
        _ => {
          let res = Some(&mut *self.start);
          self.start = match size_of::<T>() {
            0 => (self.start as usize + 1) as *mut T,
            _ => self.start.offset(1),
          };
          res
        }
      }
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = distance_between(self.end, self.start);
    (len, Some(len))
  }

  #[inline(always)]
  fn count(self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn nth(&mut self, n: usize) -> Option<&'a mut T> {
    if n >= distance_between(self.end, self.start) {
      None
    } else {
      unsafe {
        match size_of::<T>() {
          0 => {
            self.end = (self.end as *mut u8).wrapping_offset(-(n as isize)) as *mut T;
            self.start = (self.start as usize + 1) as *mut T;
            Some(&mut *self.start)
          }
          _ => {
            let res = self.start.add(n);
            self.start = res.offset(1);
            Some(&mut *res)
          }
        }
      }
    }
  }

  #[inline(always)]
  fn last(mut self) -> Option<&'a mut T> {
    self.next_back()
  }

  #[inline(always)]
  unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> &'a mut T {
    // Safety: see the comments for the implementation of this function in
    // 'core/slice/iter/macros.rs`, as they're equally applicable to us here.
    &mut *self.start.add(idx)
  }
}

impl<'a, T: 'a, const N: usize> DoubleEndedIterator for StaticVecIterMut<'a, T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<&'a mut T> {
    unsafe {
      // Safety: `self.start` and `self.end` are never null if `T` is not a ZST,
      // and the possibility that `self.end` specifically is null if `T` *is* a ZST
      // is accounted for.
      assume(!self.start.is_null());
      if size_of::<T>() != 0 {
        assume(!self.end.is_null());
      }
      match distance_between(self.end, self.start) {
        0 => None,
        _ => {
          self.end = match size_of::<T>() {
            0 => (self.end as usize - 1) as *mut T,
            _ => self.end.offset(-1),
          };
          Some(&mut *self.end)
        }
      }
    }
  }

  #[inline(always)]
  fn nth_back(&mut self, n: usize) -> Option<&'a mut T> {
    if n >= distance_between(self.end, self.start) {
      None
    } else {
      unsafe {
        self.end = match size_of::<T>() {
          0 => (self.end as *mut u8).wrapping_offset(-((n as isize) + 1)) as *mut T,
          _ => self.end.offset(-((n as isize) + 1)),
        };
        Some(&mut *self.end)
      }
    }
  }
}

impl<'a, T: 'a, const N: usize> const ExactSizeIterator for StaticVecIterMut<'a, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a, const N: usize> const FusedIterator for StaticVecIterMut<'a, T, N> {}
unsafe impl<'a, T: 'a, const N: usize> const TrustedLen for StaticVecIterMut<'a, T, N> {}
// We hide this one just in case it gets removed from `std` later, so no one relies on us relying on
// it.
#[doc(hidden)]
unsafe impl<'a, T: 'a, const N: usize> const TrustedRandomAccessNoCoerce
  for StaticVecIterMut<'a, T, N>
{
  const MAY_HAVE_SIDE_EFFECT: bool = false;
  fn size(&self) -> usize {
    distance_between(self.end, self.start)
  }
}
unsafe impl<'a, T: 'a + Sync, const N: usize> const Sync for StaticVecIterMut<'a, T, N> {}
unsafe impl<'a, T: 'a + Send, const N: usize> const Send for StaticVecIterMut<'a, T, N> {}

impl<'a, T: 'a + Debug, const N: usize> Debug for StaticVecIterMut<'a, T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("StaticVecIterMut")
      .field(&self.as_slice())
      .finish()
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
    match self.len() {
      0 => String::from("Empty iterator!"),
      _ => unsafe {
        // Safety: `start` and `end` are never out of bounds.
        format!(
          "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
          &*StaticVec::first_ptr(&self.data).add(self.start),
          &*StaticVec::first_ptr(&self.data).add(self.end - 1)
        )
      },
    }
  }

  /// Returns an immutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` indices.
  #[inline(always)]
  pub const fn as_slice(&self) -> &[T] {
    // Safety: `start_at` is never null. This function will "at worst" return an empty slice.
    let start_at = unsafe {
      match size_of::<T>() {
        0 => zst_ptr_add(StaticVec::first_ptr(&self.data), self.start),
        _ => StaticVec::first_ptr(&self.data).add(self.start),
      }
    };
    unsafe { from_raw_parts(start_at, self.end - self.start) }
  }

  /// Returns a mutable slice consisting of the elements in the range between the iterator's
  /// `start` and `end` indices.
  #[inline(always)]
  pub const fn as_mut_slice(&mut self) -> &mut [T] {
    // Safety: `start_at` is never null. This function will "at worst" return an empty slice.
    let start_at = unsafe {
      match size_of::<T>() {
        0 => zst_ptr_add_mut(StaticVec::first_ptr_mut(&mut self.data), self.start),
        _ => StaticVec::first_ptr_mut(&mut self.data).add(self.start),
      }
    };
    unsafe { from_raw_parts_mut(start_at, self.end - self.start) }
  }
}

impl<T, const N: usize> Iterator for StaticVecIntoIter<T, N> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    match self.end - self.start {
      0 => None,
      _ => {
        let res = match size_of::<T>() {
          0 => Some(unsafe { zst_ptr_add(StaticVec::first_ptr(&self.data), self.start).read() }),
          _ => Some(unsafe { StaticVec::first_ptr(&self.data).add(self.start).read() }),
        };
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

  #[inline(always)]
  fn count(self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn nth(&mut self, n: usize) -> Option<T> {
    if n >= self.len() {
      None
    } else {
      unsafe {
        let old_start = self.start;
        // Get the index in `self.data` of the item to be returned.
        let res_index = old_start + n;
        // Get a pointer to the item, using the above index.
        let res = match size_of::<T>() {
          0 => zst_ptr_add(StaticVec::first_ptr(&self.data), res_index),
          _ => StaticVec::first_ptr(&self.data).add(res_index),
        };
        // Drop whatever range of values may exist in earlier positions
        // to avoid memory leaks.
        let drop_at = match size_of::<T>() {
          0 => zst_ptr_add_mut(StaticVec::first_ptr_mut(&mut self.data), old_start),
          _ => StaticVec::first_ptr_mut(&mut self.data).add(old_start),
        };
        ptr::drop_in_place(from_raw_parts_mut(drop_at, res_index - old_start));
        // Adjust our starting index.
        self.start = res_index + 1;
        Some(res.read())
      }
    }
  }

  #[inline(always)]
  fn last(mut self) -> Option<T> {
    self.next_back()
  }

  #[inline(always)]
  unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> T
  where Self: TrustedRandomAccessNoCoerce {
    // Safety: `TrustedRandomAccessNoCoerce` is only implemented for `StaticVecIntoIter` when `T` is
    // `Copy`, and the caller is required to uphold the invariants for this function.
    match size_of::<T>() {
      0 => zst_ptr_add(StaticVec::first_ptr(&self.data), idx).read(),
      _ => StaticVec::first_ptr(&self.data).add(idx).read(),
    }
  }
}

impl<T, const N: usize> DoubleEndedIterator for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<T> {
    match self.end - self.start {
      0 => None,
      _ => {
        self.end -= 1;
        match size_of::<T>() {
          0 => Some(unsafe { zst_ptr_add(StaticVec::first_ptr(&self.data), self.end).read() }),
          _ => Some(unsafe { StaticVec::first_ptr(&self.data).add(self.end).read() }),
        }
      }
    }
  }

  #[inline(always)]
  fn nth_back(&mut self, n: usize) -> Option<T> {
    if n >= self.len() {
      None
    } else {
      let old_end = self.end;
      // Get the index in `self.data` of the item to be returned.
      let res_index = old_end - n;
      // Adjust our ending index.
      self.end = res_index - 1;
      // Drop whatever range of values may exist in later positions
      // to avoid memory leaks.
      unsafe {
        let drop_at = match size_of::<T>() {
          0 => zst_ptr_add_mut(StaticVec::first_ptr_mut(&mut self.data), res_index),
          _ => StaticVec::first_ptr_mut(&mut self.data).add(res_index),
        };
        ptr::drop_in_place(from_raw_parts_mut(drop_at, old_end - res_index));
        match size_of::<T>() {
          0 => Some(zst_ptr_add(StaticVec::first_ptr(&self.data), self.end).read()),
          _ => Some(StaticVec::first_ptr(&self.data).add(self.end).read()),
        }
      }
    }
  }
}

impl<T, const N: usize> const ExactSizeIterator for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.end - self.start
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.end - self.start == 0
  }
}

impl<T, const N: usize> const FusedIterator for StaticVecIntoIter<T, N> {}
unsafe impl<T, const N: usize> const TrustedLen for StaticVecIntoIter<T, N> {}
// We hide this one just in case it gets removed from `std` later, so no one relies on us relying on
// it.
#[doc(hidden)]
unsafe impl<T: Copy, const N: usize> const TrustedRandomAccessNoCoerce for StaticVecIntoIter<T, N> {
  const MAY_HAVE_SIDE_EFFECT: bool = false;

  fn size(&self) -> usize {
    self.end - self.start
  }
}
unsafe impl<T: Sync, const N: usize> const Sync for StaticVecIntoIter<T, N> {}
unsafe impl<T: Send, const N: usize> const Send for StaticVecIntoIter<T, N> {}

impl<T: Clone, const N: usize> Clone for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn clone(&self) -> StaticVecIntoIter<T, N> {
    Self {
      start: self.start,
      end: self.end,
      data: {
        let mut data = MaybeUninit::<[T; N]>::uninit();
        let new_data_ptr = data.as_mut_ptr() as *mut T;
        let self_data_ptr = self.data.as_ptr() as *const T;
        unsafe {
          // These are guaranteed safe assumptions in this context.
          assume(!new_data_ptr.is_null());
          assume(!self_data_ptr.is_null());
          for i in self.start..self.end {
            new_data_ptr.add(i).write((&*self_data_ptr.add(i)).clone());
          }
        }
        data
      },
    }
  }
}

impl<T: Debug, const N: usize> Debug for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("StaticVecIntoIter")
      .field(&self.as_slice())
      .finish()
  }
}

impl<T, const N: usize> Drop for StaticVecIntoIter<T, N> {
  #[inline(always)]
  fn drop(&mut self) {
    let item_count = self.end - self.start;
    let drop_at = match size_of::<T>() {
      0 => zst_ptr_add_mut(StaticVec::first_ptr_mut(&mut self.data), self.start),
      _ => unsafe { StaticVec::first_ptr_mut(&mut self.data).add(self.start) },
    };
    match item_count {
      0 => (),
      _ => unsafe { ptr::drop_in_place(from_raw_parts_mut(drop_at, item_count)) },
    }
  }
}

impl<'a, T: 'a, const N: usize> StaticVecDrain<'a, T, N> {
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    self.iter.bounds_to_string()
  }

  /// Returns an immutable slice consisting of the current range of elements the iterator has a view
  /// over.
  #[inline(always)]
  pub const fn as_slice(&self) -> &[T] {
    self.iter.as_slice()
  }
}

impl<'a, T: 'a, const N: usize> Iterator for StaticVecDrain<'a, T, N> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    self
      .iter
      .next()
      .map(|val| unsafe { (val as *const T).read() })
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }

  #[inline(always)]
  fn count(self) -> usize {
    self.len()
  }

  #[inline(always)]
  fn last(mut self) -> Option<T> {
    self.next_back()
  }

  #[inline(always)]
  unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> T
  where Self: TrustedRandomAccessNoCoerce {
    // Safety: `TrustedRandomAccessNoCoerce` is only implemented for `StaticVecDrain` when `T` is
    // `Copy`, and the caller is required to uphold the invariants for this function.
    match size_of::<T>() {
      0 => zst_ptr_add(self.as_slice().as_ptr(), idx).read(),
      _ => self.as_slice().as_ptr().add(idx).read(),
    }
  }
}

impl<'a, T: 'a, const N: usize> DoubleEndedIterator for StaticVecDrain<'a, T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<T> {
    self
      .iter
      .next_back()
      .map(|val| unsafe { (val as *const T).read() })
  }
}

impl<'a, T: 'a, const N: usize> const ExactSizeIterator for StaticVecDrain<'a, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.iter.len()
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.iter.is_empty()
  }
}

impl<'a, T: 'a, const N: usize> const FusedIterator for StaticVecDrain<'a, T, N> {}
unsafe impl<'a, T: 'a, const N: usize> const TrustedLen for StaticVecDrain<'a, T, N> {}
// We hide this one just in case it gets removed from `std` later, so no one relies on us relying on
// it.
#[doc(hidden)]
unsafe impl<'a, T: Copy + 'a, const N: usize> const TrustedRandomAccessNoCoerce
  for StaticVecDrain<'a, T, N>
{
  const MAY_HAVE_SIDE_EFFECT: bool = false;
  fn size(&self) -> usize {
    distance_between(self.iter.end, self.iter.start)
  }
}
unsafe impl<'a, T: 'a + Sync, const N: usize> const Sync for StaticVecDrain<'a, T, N> {}
unsafe impl<'a, T: 'a + Send, const N: usize> const Send for StaticVecDrain<'a, T, N> {}

impl<'a, T: 'a + Debug, const N: usize> Debug for StaticVecDrain<'a, T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("StaticVecDrain")
      .field(&self.iter.as_slice())
      .finish()
  }
}

impl<'a, T: 'a, const N: usize> Drop for StaticVecDrain<'a, T, N> {
  #[inline]
  fn drop(&mut self) {
    // Handle ZST edge case
    if size_of::<T>() == 0 {
      if self.iter.start.is_null() || self.iter.end.is_null() {
        return;
      }
    }
    // Read out any remaining contents first.
    while let Some(_) = self.next() {}
    // Adjust the StaticVec that this StaticVecDrain was created from, if necessary.
    let total_length = self.length;
    if total_length > 0 {
      unsafe {
        let vec_ref = &mut *self.vec;
        let start = vec_ref.length;
        let tail = self.start;
        vec_ref
          .ptr_at_unchecked(tail)
          .copy_to(vec_ref.mut_ptr_at_unchecked(start), total_length);
        vec_ref.set_len(start + total_length);
      }
    }
  }
}

impl<T, I: Iterator<Item = T>, const N: usize> Iterator for StaticVecSplice<T, I, N> {
  type Item = T;

  #[inline]
  fn next(&mut self) -> Option<T> {
    // We contextually already know we're within an appropriate range, so bounds checking
    // is not necessary for any of the `self.vec` method calls below.
    match self.end - self.start {
      0 => None,
      _ => match self.replace_with.next() {
        Some(replace_with) => unsafe {
          let removed = replace((&mut *self.vec).get_unchecked_mut(self.start), replace_with);
          self.start += 1;
          Some(removed)
        },
        None => unsafe {
          let removed = (&mut *self.vec).remove_unchecked(self.start);
          self.end -= 1;
          Some(removed)
        },
      },
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.len();
    (len, Some(len))
  }

  #[inline(always)]
  fn count(self) -> usize {
    self.len()
  }
}

impl<T, I: Iterator<Item = T> + DoubleEndedIterator, const N: usize> DoubleEndedIterator
  for StaticVecSplice<T, I, N>
{
  #[inline]
  fn next_back(&mut self) -> Option<T> {
    // We contextually already know we're within an appropriate range, so bounds checking
    // is not necessary for any of the `self.vec` method calls below.
    match self.end - self.start {
      0 => None,
      _ => match self.replace_with.next_back() {
        Some(replace_with) => unsafe {
          let removed = replace(
            (&mut *self.vec).get_unchecked_mut(self.end - 1),
            replace_with,
          );
          self.end -= 1;
          Some(removed)
        },
        None => unsafe {
          let removed = (&mut *self.vec).remove_unchecked(self.end - 1);
          self.end -= 1;
          Some(removed)
        },
      },
    }
  }
}

impl<T, I: Iterator<Item = T>, const N: usize> const ExactSizeIterator
  for StaticVecSplice<T, I, N>
{
  #[inline(always)]
  fn len(&self) -> usize {
    self.end - self.start
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.end - self.start == 0
  }
}

impl<T, I: Iterator<Item = T>, const N: usize> const FusedIterator for StaticVecSplice<T, I, N> {}
unsafe impl<T, I: Iterator<Item = T>, const N: usize> const TrustedLen
  for StaticVecSplice<T, I, N>
{
}
unsafe impl<T: Sync, I: Iterator<Item = T>, const N: usize> const Sync
  for StaticVecSplice<T, I, N>
{
}
unsafe impl<T: Send, I: Iterator<Item = T>, const N: usize> const Send
  for StaticVecSplice<T, I, N>
{
}

impl<T: Debug, I: Iterator<Item = T>, const N: usize> Debug for StaticVecSplice<T, I, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    // Safety: `self.vec` will never be null, and `self.end` and `self.start` will always be within
    // an appropriate range.
    unsafe {
      let items = from_raw_parts(
        (&*self.vec).ptr_at_unchecked(self.start),
        self.end - self.start,
      );
      f.debug_tuple("StaticVecSplice").field(&items).finish()
    }
  }
}

impl<T, I: Iterator<Item = T>, const N: usize> Drop for StaticVecSplice<T, I, N> {
  #[inline]
  fn drop(&mut self) {
    while let Some(_) = self.next() {}
    let vec_ref = unsafe { &mut *self.vec };
    for replace_with in self.replace_with.by_ref() {
      // Stop looping if the StaticVec is at maximum capacity.
      let old_length = vec_ref.length;
      if old_length == N {
        break;
      }
      // The next bit is just the code from `StaticVec::insert`, without the initial bounds check
      // since we already know that we're within an appropriate range in this context.
      let index = self.end;
      unsafe {
        let vec_ptr = vec_ref.mut_ptr_at_unchecked(index);
        vec_ptr.copy_to(vec_ptr.offset(1), old_length - index);
        vec_ptr.write(replace_with);
        vec_ref.set_len(old_length + 1);
      }
      self.end += 1;
    }
  }
}
