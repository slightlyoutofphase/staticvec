use super::StaticHeap;
use core::fmt::{self, Debug, Formatter};
use core::iter::{FusedIterator, TrustedLen};

/// A sorted "consuming" iterator over the elements of a [`StaticHeap`].
///
/// This struct is created by the [`into_iter_sorted`] method on [`StaticHeap`]. See its
/// documentation for more.
///
/// [`into_iter_sorted`]: struct.StaticHeap.html#method.into_iter_sorted
/// [`StaticHeap`]: struct.StaticHeap.html
#[derive(Clone)]
pub struct StaticHeapIntoIterSorted<T, const N: usize> {
  pub(crate) inner: StaticHeap<T, N>,
}

/// A sorted "draining" iterator over the elements of a [`StaticHeap`].
///
/// This struct is created by the [`drain_sorted`] method on [`StaticHeap`]. See its
/// documentation for more.
///
/// [`drain_sorted`]: struct.StaticHeap.html#method.drain_sorted
/// [`StaticHeap`]: struct.StaticHeap.html
pub struct StaticHeapDrainSorted<'a, T: Ord, const N: usize> {
  pub(crate) inner: &'a mut StaticHeap<T, N>,
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
  
  #[inline(always)]
  fn count(self) -> usize {
    self.len()
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
unsafe impl<T: Ord + Send, const N: usize> Send for StaticHeapIntoIterSorted<T, N> {}

impl<T: Debug, const N: usize> Debug for StaticHeapIntoIterSorted<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("StaticHeapIntoIterSorted")
      .field(&self.inner.data.as_slice())
      .finish()
  }
}

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
  
  #[inline(always)]
  fn count(self) -> usize {
    self.len()
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
unsafe impl<T: Ord + Send, const N: usize> Send for StaticHeapDrainSorted<'_, T, N> {}

impl<'a, T: 'a + Ord + Debug, const N: usize> Debug for StaticHeapDrainSorted<'a, T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("StaticHeapDrainSorted")
      .field(&self.inner.data.as_slice())
      .finish()
  }
}

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
