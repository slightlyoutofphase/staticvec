use super::StaticHeap;
use crate::iterators::{StaticVecDrain, StaticVecIntoIter, StaticVecIterConst};
use core::fmt::{self, Debug, Formatter};
use core::iter::{FusedIterator, TrustedLen};

/// An iterator over the elements of a [`StaticHeap`].
///
/// This `struct` is created by the [`iter`] method on [`StaticHeap`]. See its
/// documentation for more.
///
/// [`iter`]: struct.StaticHeap.html#method.iter
/// [`StaticHeap`]: struct.StaticHeap.html
pub struct StaticHeapIter<'a, T: 'a, const N: usize> {
  pub(crate) iter: StaticVecIterConst<'a, T, N>,
}

/// A "consuming" iterator over the elements of a [`StaticHeap`].
///
/// This `struct` is created by the [`into_iter`] method on [`StaticHeap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: struct.StaticHeap.html#method.into_iter
/// [`StaticHeap`]: struct.StaticHeap.html
#[derive(Clone)]
pub struct StaticHeapIntoIter<T, const N: usize> {
  pub(crate) iter: StaticVecIntoIter<T, N>,
}

/// A sorted "consuming" iterator over the elements of a [`StaticHeap`].
#[derive(Clone, Debug)]
pub struct StaticHeapIntoIterSorted<T, const N: usize> {
  pub(crate) inner: StaticHeap<T, N>,
}

/// A "draining" iterator over the elements of a [`StaticHeap`].
///
/// This `struct` is created by the [`drain`] method on [`StaticHeap`]. See its
/// documentation for more.
///
/// [`drain`]: struct.StaticHeap.html#method.drain
/// [`StaticHeap`]: struct.StaticHeap.html
#[derive(Debug)]
pub struct StaticHeapDrain<'a, T: 'a, const N: usize> {
  pub(crate) iter: StaticVecDrain<'a, T, N>,
}

/// A sorted "draining" iterator over the elements of a [`StaticHeap`].
///
/// This `struct` is created by the [`drain_sorted`] method on [`StaticHeap`]. See its
/// documentation for more.
///
/// [`drain_sorted`]: struct.StaticHeap.html#method.drain_sorted
/// [`StaticHeap`]: struct.StaticHeap.html
#[derive(Debug)]
pub struct StaticHeapDrainSorted<'a, T: Ord, const N: usize> {
  pub(crate) inner: &'a mut StaticHeap<T, N>,
}

impl<'a, T, const N: usize> Iterator for StaticHeapIter<'a, T, N> {
  type Item = &'a T;

  #[inline(always)]
  fn next(&mut self) -> Option<&'a T> {
    self.iter.next()
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }

  #[inline(always)]
  fn last(self) -> Option<&'a T> {
    self.iter.last()
  }
}

impl<'a, T, const N: usize> DoubleEndedIterator for StaticHeapIter<'a, T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<&'a T> {
    self.iter.next_back()
  }
}

impl<T, const N: usize> ExactSizeIterator for StaticHeapIter<'_, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.iter.len()
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.iter.is_empty()
  }
}

impl<T, const N: usize> FusedIterator for StaticHeapIter<'_, T, N> {}
unsafe impl<T, const N: usize> TrustedLen for StaticHeapIter<'_, T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for StaticHeapIter<'_, T, N> {}
unsafe impl<T: Sync, const N: usize> Send for StaticHeapIter<'_, T, N> {}

impl<T, const N: usize> Clone for StaticHeapIter<'_, T, N> {
  #[inline(always)]
  fn clone(&self) -> Self {
    StaticHeapIter {
      iter: self.iter.clone(),
    }
  }
}

impl<T: Debug, const N: usize> Debug for StaticHeapIter<'_, T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("StaticHeapIter")
      .field(&self.iter.as_slice())
      .finish()
  }
}

impl<T, const N: usize> Iterator for StaticHeapIntoIter<T, N> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    self.iter.next()
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
}

impl<T, const N: usize> DoubleEndedIterator for StaticHeapIntoIter<T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<T> {
    self.iter.next_back()
  }
}

impl<T, const N: usize> ExactSizeIterator for StaticHeapIntoIter<T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.iter.len()
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.iter.is_empty()
  }
}

impl<T, const N: usize> FusedIterator for StaticHeapIntoIter<T, N> {}
unsafe impl<T, const N: usize> TrustedLen for StaticHeapIntoIter<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for StaticHeapIntoIter<T, N> {}
unsafe impl<T: Sync, const N: usize> Send for StaticHeapIntoIter<T, N> {}

impl<T: Debug, const N: usize> Debug for StaticHeapIntoIter<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("StaticHeapIntoIter")
      .field(&self.iter.as_slice())
      .finish()
  }
}

impl<T: Ord, const N: usize> Iterator for StaticHeapIntoIterSorted<T, N> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    self.inner.pop()
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let exact = self.inner.len();
    (exact, Some(exact))
  }
}

impl<T: Ord, const N: usize> ExactSizeIterator for StaticHeapIntoIterSorted<T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.inner.len()
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}

impl<T: Ord, const N: usize> FusedIterator for StaticHeapIntoIterSorted<T, N> {}
unsafe impl<T: Ord, const N: usize> TrustedLen for StaticHeapIntoIterSorted<T, N> {}
unsafe impl<T: Ord + Sync, const N: usize> Sync for StaticHeapIntoIterSorted<T, N> {}
unsafe impl<T: Ord + Sync, const N: usize> Send for StaticHeapIntoIterSorted<T, N> {}

impl<T, const N: usize> Iterator for StaticHeapDrain<'_, T, N> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    self.iter.next()
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
}

impl<T, const N: usize> DoubleEndedIterator for StaticHeapDrain<'_, T, N> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<T> {
    self.iter.next_back()
  }
}

impl<T, const N: usize> ExactSizeIterator for StaticHeapDrain<'_, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.iter.len()
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.iter.is_empty()
  }
}

impl<T, const N: usize> FusedIterator for StaticHeapDrain<'_, T, N> {}
unsafe impl<T, const N: usize> TrustedLen for StaticHeapDrain<'_, T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for StaticHeapDrain<'_, T, N> {}
unsafe impl<T: Sync, const N: usize> Send for StaticHeapDrain<'_, T, N> {}

impl<T: Ord, const N: usize> Iterator for StaticHeapDrainSorted<'_, T, N> {
  type Item = T;

  #[inline(always)]
  fn next(&mut self) -> Option<T> {
    self.inner.pop()
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let exact = self.inner.len();
    (exact, Some(exact))
  }
}

impl<T: Ord, const N: usize> ExactSizeIterator for StaticHeapDrainSorted<'_, T, N> {
  #[inline(always)]
  fn len(&self) -> usize {
    self.inner.len()
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}

impl<T: Ord, const N: usize> FusedIterator for StaticHeapDrainSorted<'_, T, N> {}
unsafe impl<T: Ord, const N: usize> TrustedLen for StaticHeapDrainSorted<'_, T, N> {}
unsafe impl<T: Ord + Sync, const N: usize> Sync for StaticHeapDrainSorted<'_, T, N> {}
unsafe impl<T: Ord + Sync, const N: usize> Send for StaticHeapDrainSorted<'_, T, N> {}

impl<'a, T: Ord, const N: usize> Drop for StaticHeapDrainSorted<'a, T, N> {
  /// Removes heap elements in heap order.
  #[inline]
  fn drop(&mut self) {
    struct DropGuard<'r, 'a, T: Ord, const N: usize>(&'r mut StaticHeapDrainSorted<'a, T, N>);
    impl<'r, 'a, T: Ord, const N: usize> Drop for DropGuard<'r, 'a, T, N> {
      #[inline(always)]
      fn drop(&mut self) {
        while let Some(_) = self.0.inner.pop() {}
      }
    }
    while let Some(item) = self.inner.pop() {
      let guard = DropGuard(self);
      drop(item);
      core::mem::forget(guard);
    }
  }
}
