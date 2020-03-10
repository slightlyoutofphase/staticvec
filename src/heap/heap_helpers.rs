use core::fmt::{self, Debug, Formatter};
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::StaticHeap;

/// A struct wrapping a mutable reference to the greatest (or "maximal") item in a [`StaticHeap`].
///
/// This struct is created by the [`peek_mut`] method on [`StaticHeap`]. See
/// its documentation for more.
///
/// [`peek_mut`]: struct.StaticHeap.html#method.peek_mut
/// [`StaticHeap`]: struct.StaticHeap.html
pub struct StaticHeapPeekMut<'a, T: 'a + Ord, const N: usize> {
  pub(crate) heap: &'a mut StaticHeap<T, N>,
  pub(crate) sift: bool,
}

/// ['StaticHeapHole'] represents a hole in a slice i.e., an index without valid value
/// (because it was moved from or duplicated).
/// In drop, `StaticHeapHole` will restore the slice by filling the hole
/// position with the value that was originally removed.
pub(crate) struct StaticHeapHole<'a, T: 'a> {
  pub(crate) data: &'a mut [T],
  pub(crate) element: ManuallyDrop<T>,
  pub(crate) position: usize,
}

impl<'a, T: Ord, const N: usize> StaticHeapPeekMut<'a, T, N> {
  /// Removes the peeked value from the heap and returns it.
  #[inline(always)]
  pub fn pop(mut this: StaticHeapPeekMut<'a, T, N>) -> T {
    let value = this.heap.pop().unwrap();
    this.sift = false;
    value
  }
}

impl<T: Ord + Debug, const N: usize> Debug for StaticHeapPeekMut<'_, T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    unsafe {
      // Safe: StaticHeapPeekMut is only instantiated for non-empty heaps
      f.debug_tuple("StaticHeapPeekMut")
        .field(self.heap.data.get_unchecked(0))
        .finish()
    }
  }
}

impl<T: Ord, const N: usize> Deref for StaticHeapPeekMut<'_, T, N> {
  type Target = T;

  #[inline(always)]
  fn deref(&self) -> &T {
    debug_assert!(self.heap.is_not_empty());
    // Safe: StaticHeapPeekMut is only instantiated for non-empty heaps
    unsafe { self.heap.data.get_unchecked(0) }
  }
}

impl<T: Ord, const N: usize> DerefMut for StaticHeapPeekMut<'_, T, N> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut T {
    debug_assert!(self.heap.is_not_empty());
    // Safe: StaticHeapPeekMut is only instantiated for non-empty heaps.
    unsafe { self.heap.data.get_unchecked_mut(0) }
  }
}

impl<T: Ord, const N: usize> Drop for StaticHeapPeekMut<'_, T, N> {
  #[inline(always)]
  fn drop(&mut self) {
    if self.sift {
      self.heap.sift_down_range(0, self.heap.len());
    }
  }
}

impl<'a, T> StaticHeapHole<'a, T> {
  /// Create a new `StaticHeapHole` at index `position`.
  /// Unsafe because position must be within the data slice.
  #[inline(always)]
  pub(crate) unsafe fn new(data: &'a mut [T], position: usize) -> Self {
    debug_assert!(position < data.len());
    let element = data.as_ptr().add(position).read();
    StaticHeapHole {
      data,
      // Safe: position should be inside the slice.
      element: ManuallyDrop::new(element),
      position,
    }
  }

  #[inline(always)]
  pub(crate) const fn pos(&self) -> usize {
    self.position
  }

  /// Returns a reference to the element removed.
  #[inline(always)]
  pub(crate) fn elt(&self) -> &T {
    &self.element
  }

  /// Returns a reference to the element at `index`.
  /// Unsafe because `index` must be within the data slice and not equal to `position`.
  #[inline(always)]
  pub(crate) unsafe fn get(&self, index: usize) -> &T {
    debug_assert!(index != self.position);
    debug_assert!(index < self.data.len());
    self.data.get_unchecked(index)
  }

  /// Move the StaticHeapHole to a new location.
  /// Unsafe because index must be within the data slice and not equal to position.
  #[inline]
  pub(crate) unsafe fn move_to(&mut self, index: usize) {
    debug_assert!(index != self.position);
    debug_assert!(index < self.data.len());
    let index_ptr = self.data.as_ptr().add(index);
    let hole_ptr = self.data.as_mut_ptr().add(self.position);
    index_ptr.copy_to_nonoverlapping(hole_ptr, 1);
    self.position = index;
  }
}

impl<T> Drop for StaticHeapHole<'_, T> {
  #[inline(always)]
  fn drop(&mut self) {
    // fill the hole again
    unsafe {
      let position = self.position;
      let element_ptr = &*self.element as *const T;
      let hole_ptr = self.data.as_mut_ptr().add(position);
      element_ptr.copy_to_nonoverlapping(hole_ptr, 1);
    }
  }
}
