use self::{heap_helpers::*, heap_iterators::*};
use crate::StaticVec;
use core::intrinsics::size_of;
use core::mem::swap;

pub mod heap_helpers;
pub mod heap_iterators;
mod heap_trait_impls;

/// A priority queue implemented with a binary heap, built around an instance of `StaticVec<T, N>`.
/// `StaticHeap`, as well as the associated iterator and helper structs for it are direct
/// adaptations of the ones found in the `std::collections::binary_heap` module (including the
/// documentation).
///
/// It is a logic error for an item to be modified in such a way that the
/// item's ordering relative to any other item, as determined by the `Ord`
/// trait, changes while it is in the heap. This is normally only possible
/// through `Cell`, `RefCell`, global state, I/O, or unsafe code.
///
/// # Examples
///
/// ```
/// use staticvec::StaticHeap;
///
/// let mut heap = StaticHeap::<i32, 4>::new();
///
/// // We can use peek to look at the next item in the heap. In this case,
/// // there's no items in there yet so we get None.
/// assert_eq!(heap.peek(), None);
///
/// // Let's add some scores...
/// heap.push(1);
/// heap.push(5);
/// heap.push(2);
///
/// // Now peek shows the most important item in the heap.
/// assert_eq!(heap.peek(), Some(&5));
///
/// // We can check the length of a heap.
/// assert_eq!(heap.len(), 3);
///
/// // We can iterate over the items in the heap, although they are returned in
/// // a random order.
/// for x in &heap {
///   println!("{}", x);
/// }
///
/// // If we instead pop these scores, they should come back in order.
/// assert_eq!(heap.pop(), Some(5));
/// assert_eq!(heap.pop(), Some(2));
/// assert_eq!(heap.pop(), Some(1));
/// assert_eq!(heap.pop(), None);
///
/// // We can clear the heap of any remaining items.
/// heap.clear();
///
/// // The heap should now be empty.
/// assert!(heap.is_empty())
/// ```
///
/// ## Min-heap
///
/// Either `core::cmp::Reverse` or a custom `Ord` implementation can be used to
/// make `StaticHeap` a min-heap. This makes `heap.pop()` return the smallest
/// value instead of the greatest one.
///
/// ```
/// use staticvec::{staticvec, StaticHeap};
/// use core::cmp::Reverse;
///
/// // Wrap the values in `Reverse`.
/// let mut heap = StaticHeap::from(staticvec![Reverse(1), Reverse(5), Reverse(2)]);
///
/// // If we pop these scores now, they should come back in the reverse order.
/// assert_eq!(heap.pop(), Some(Reverse(1)));
/// assert_eq!(heap.pop(), Some(Reverse(2)));
/// assert_eq!(heap.pop(), Some(Reverse(5)));
/// assert_eq!(heap.pop(), None);
/// ```
///
/// # Time complexity
///
/// | [push] | [pop]    | [peek]/[peek\_mut] |
/// |--------|----------|--------------------|
/// | O(1)~  | O(log n) | O(1)               |
///
/// The value for `push` is an expected cost; the method documentation gives a
/// more detailed analysis.
///
/// [push]: #method.push
/// [pop]: #method.pop
/// [peek]: #method.peek
/// [peek\_mut]: #method.peek_mut
pub struct StaticHeap<T, const N: usize> {
  pub(crate) data: StaticVec<T, N>,
}

impl<T: Ord, const N: usize> StaticHeap<T, N> {
  /// Creates an empty `StaticHeap` as a max-heap.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::StaticHeap;
  /// let mut heap = StaticHeap::<i32, 12>::new();
  /// heap.push(4);
  /// ```
  #[inline(always)]
  pub const fn new() -> StaticHeap<T, N> {
    StaticHeap {
      data: StaticVec::new(),
    }
  }

  /// Returns a mutable reference to the greatest item in the binary heap, or
  /// `None` if it is empty.
  ///
  /// Note: If the `StaticHeapPeekMut` value is leaked, the heap may be in an
  /// inconsistent state.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::StaticHeap;
  /// let mut heap = StaticHeap::<i32, 12>::new();
  /// assert!(heap.peek_mut().is_none());
  /// heap.push(1);
  /// heap.push(5);
  /// heap.push(2);
  /// {
  ///   let mut val = heap.peek_mut().unwrap();
  ///   *val = 0;
  /// }
  /// assert_eq!(heap.peek(), Some(&2));
  /// ```
  ///
  /// # Time complexity
  ///
  /// Cost is O(1) in the worst case.
  #[inline(always)]
  pub fn peek_mut(&mut self) -> Option<StaticHeapPeekMut<'_, T, N>> {
    if self.is_empty() {
      None
    } else {
      Some(StaticHeapPeekMut {
        heap: self,
        sift: true,
      })
    }
  }

  /// Removes the greatest item from the binary heap and returns it, or `None` if it
  /// is empty.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::from(staticvec![1, 3]);
  /// assert_eq!(heap.pop(), Some(3));
  /// assert_eq!(heap.pop(), Some(1));
  /// assert_eq!(heap.pop(), None);
  /// ```
  ///
  /// # Time complexity
  ///
  /// The worst case cost of `pop` on a heap containing *n* elements is O(log n).
  #[inline(always)]
  pub fn pop(&mut self) -> Option<T> {
    self.data.pop().map(|mut item| {
      if !self.is_empty() {
        swap(&mut item, unsafe { self.data.get_unchecked_mut(0) });
        self.sift_down_to_bottom(0);
      }
      item
    })
  }

  /// Pushes an item onto the binary heap.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::StaticHeap;
  /// let mut heap = StaticHeap::<i32, 12>::new();
  /// heap.push(3);
  /// heap.push(5);
  /// heap.push(1);
  /// assert_eq!(heap.len(), 3);
  /// assert_eq!(heap.peek(), Some(&5));
  /// ```
  ///
  /// # Time complexity
  ///
  /// The expected cost of `push`, averaged over every possible ordering of
  /// the elements being pushed, and over a sufficiently large number of
  /// pushes, is O(1). This is the most meaningful cost metric when pushing
  /// elements that are *not* already in any sorted pattern.
  ///
  /// The time complexity degrades if elements are pushed in predominantly
  /// ascending order. In the worst case, elements are pushed in ascending
  /// sorted order and the amortized cost per push is O(log n) against a heap
  /// containing *n* elements.
  ///
  /// The worst case cost of a *single* call to `push` is O(n). The worst case
  /// occurs when capacity is exhausted and needs a resize. The resize cost
  /// has been amortized in the previous figures.
  #[inline(always)]
  pub fn push(&mut self, item: T) {
    let old_len = self.len();
    self.data.push(item);
    self.sift_up(0, old_len);
  }

  /// Consumes the `StaticHeap` and returns a vector in sorted (ascending) order.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::from(StaticVec::<i32, 8>::from([1, 2, 4, 5, 7]));
  /// heap.push(6);
  /// heap.push(3);
  /// let vec = heap.into_sorted_vec();
  /// assert_eq!(vec, [1, 2, 3, 4, 5, 6, 7]);
  /// ```
  #[inline]
  pub fn into_sorted_vec(mut self) -> StaticVec<T, N> {
    let mut end = self.len();
    while end > 1 {
      end -= 1;
      self.data.swap(0, end);
      self.sift_down_range(0, end);
    }
    self.into_vec()
  }

  // The implementations of sift_up and sift_down use unsafe blocks in
  // order to move an element out of the vector (leaving behind a
  // hole), shift along the others and move the removed element back into the
  // vector at the final location of the hole.
  // The `StaticHeapHole` type is used to represent this, and make sure
  // the hole is filled back at the end of its scope, even on panic.
  // Using a hole reduces the constant factor compared to using swaps,
  // which involves twice as many moves.
  #[inline]
  fn sift_up(&mut self, start: usize, position: usize) -> usize {
    unsafe {
      // Take out the value at `position` and create a hole.
      let mut hole = StaticHeapHole::new(&mut self.data, position);
      while hole.pos() > start {
        let parent = (hole.pos() - 1) / 2;
        if hole.element() <= hole.get(parent) {
          break;
        }
        hole.move_to(parent);
      }
      hole.pos()
    }
  }

  /// Take an element at `position` and move it down the heap,
  /// while its children are larger.
  #[inline]
  fn sift_down_range(&mut self, position: usize, end: usize) {
    unsafe {
      let mut hole = StaticHeapHole::new(&mut self.data, position);
      let mut child = 2 * position + 1;
      while child < end {
        let right = child + 1;
        // compare with the greater of the two children
        if right < end && hole.get(child) <= hole.get(right) {
          child = right;
        }
        // if we are already in order, stop.
        if hole.element() >= hole.get(child) {
          break;
        }
        hole.move_to(child);
        child = 2 * hole.pos() + 1;
      }
    }
  }

  #[inline(always)]
  fn sift_down(&mut self, position: usize) {
    let len = self.len();
    self.sift_down_range(position, len);
  }

  /// Take an element at `position` and move it all the way down the heap,
  /// then sift it up to its position.
  ///
  /// Note: This is faster when the element is known to be large / should
  /// be closer to the bottom.
  #[inline]
  fn sift_down_to_bottom(&mut self, mut position: usize) {
    let end = self.len();
    let start = position;
    unsafe {
      let mut hole = StaticHeapHole::new(&mut self.data, position);
      let mut child = 2 * position + 1;
      while child < end {
        let right = child + 1;
        // compare with the greater of the two children
        if right < end && hole.get(child) <= hole.get(right) {
          child = right;
        }
        hole.move_to(child);
        child = 2 * hole.pos() + 1;
      }
      position = hole.position;
    }
    self.sift_up(start, position);
  }

  #[inline(always)]
  fn rebuild(&mut self) {
    let mut n = self.len() / 2;
    while n > 0 {
      n -= 1;
      self.sift_down(n);
    }
  }

  /// Moves all the elements of `other` into `self`, leaving `other` empty.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let v = staticvec![-10, 1, 2];
  /// let mut a = StaticHeap::<i32, 6>::from(v);
  /// let v = staticvec![-20, 5, 43];
  /// let mut b = StaticHeap::from(v);
  /// a.append(&mut b);
  /// assert_eq!(a.into_sorted_vec(), [-20, -10, 1, 2, 5, 43]);
  /// assert!(b.is_empty());
  /// ```
  #[inline]
  pub fn append(&mut self, other: &mut Self) {
    if self.len() < other.len() {
      swap(self, other);
    }

    if other.is_empty() {
      return;
    }

    #[inline(always)]
    fn log2_fast(x: usize) -> usize {
      8 * size_of::<usize>() - (x.leading_zeros() as usize) - 1
    }

    // `rebuild` takes O(len1 + len2) operations
    // and about 2 * (len1 + len2) comparisons in the worst case
    // while `extend` takes O(len2 * log_2(len1)) operations
    // and about 1 * len2 * log_2(len1) comparisons in the worst case,
    // assuming len1 >= len2.
    #[inline]
    fn better_to_rebuild(len1: usize, len2: usize) -> bool {
      2 * (len1 + len2) < len2 * log2_fast(len1)
    }

    if better_to_rebuild(self.len(), other.len()) {
      self.data.append(&mut other.data);
      self.rebuild();
    } else {
      self.extend(other.drain());
    }
  }

  /// Returns an iterator which retrieves elements in heap order.
  /// The retrieved elements are removed from the original heap.
  /// The remaining elements will be removed on drop in heap order.
  ///
  /// Note:
  /// * `drain_sorted()` is O(n log n); much slower than `drain()`. You should use the latter for
  ///   most cases.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::from(staticvec![1, 2, 3, 4, 5]);
  /// assert_eq!(heap.len(), 5);
  /// drop(heap.drain_sorted()); // removes all elements in heap order
  /// assert_eq!(heap.len(), 0);
  /// ```
  #[inline(always)]
  pub fn drain_sorted(&mut self) -> StaticHeapDrainSorted<'_, T, N> {
    StaticHeapDrainSorted { inner: self }
  }
}

impl<T, const N: usize> StaticHeap<T, N> {
  /// Returns an iterator visiting all values in the underlying vector, in
  /// arbitrary order.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let heap = StaticHeap::from(staticvec![1, 2, 3, 4]);
  /// // Print 1, 2, 3, 4 in arbitrary order
  /// for x in heap.iter() {
  ///   println!("{}", x);
  /// }
  /// ```
  #[inline(always)]
  pub fn iter(&self) -> StaticVecIterConst<'_, T, N> {
    self.data.iter()
  }

  /// Returns an iterator which retrieves elements in heap order.
  /// This method consumes the original heap.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let heap = StaticHeap::from(staticvec![1, 2, 3, 4, 5]);
  /// assert_eq!(
  ///   heap.into_iter_sorted().take(2).collect::<StaticVec<_, 3>>(), staticvec![5, 4]
  /// );
  /// ```
  #[inline(always)]
  pub fn into_iter_sorted(self) -> StaticHeapIntoIterSorted<T, N> {
    StaticHeapIntoIterSorted { inner: self }
  }

  /// Returns the greatest item in the binary heap, or `None` if it is empty.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::<i32, 12>::new();
  /// assert_eq!(heap.peek(), None);
  /// heap.push(1);
  /// heap.push(5);
  /// heap.push(2);
  /// assert_eq!(heap.peek(), Some(&5));
  /// ```
  ///
  /// # Time complexity
  ///
  /// Cost is O(1) in the worst case.
  #[inline(always)]
  pub fn peek(&self) -> Option<&T> {
    self.data.get(0)
  }

  /// Returns the number of elements the binary heap can hold without reallocating.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::<i32, 100>::new();
  /// assert!(heap.capacity() >= 100);
  /// heap.push(4);
  /// ```
  #[inline(always)]
  pub const fn capacity(&self) -> usize {
    self.data.capacity()
  }

  /// Consumes the `StaticHeap` and returns the underlying vector
  /// in arbitrary order.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let heap = StaticHeap::from(staticvec![1, 2, 3, 4, 5, 6, 7]);
  /// let vec = heap.into_vec();
  /// // Will print in some order
  /// for x in &vec {
  ///   println!("{}", x);
  /// }
  /// ```
  #[inline(always)]
  pub fn into_vec(self) -> StaticVec<T, N> {
    self.into()
  }

  /// Returns the length of the binary heap.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let heap = StaticHeap::from(staticvec![1, 3]);
  /// assert_eq!(heap.len(), 2);
  /// ```
  #[inline(always)]
  pub const fn len(&self) -> usize {
    self.data.len()
  }

  /// Checks if the StaticHeap is empty.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::<i32, 12>::new();
  /// assert!(heap.is_empty());
  /// heap.push(3);
  /// heap.push(5);
  /// heap.push(1);
  /// assert!(!heap.is_empty());
  /// ```
  #[inline(always)]
  pub const fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Clears the StaticHeap, returning an iterator over the removed elements.
  ///
  /// The elements are removed in arbitrary order.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::from(staticvec![1, 3]);
  /// assert!(!heap.is_empty());
  /// for x in heap.drain() {
  ///   println!("{}", x);
  /// }
  /// assert!(heap.is_empty());
  /// ```
  #[inline(always)]
  pub fn drain(&mut self) -> StaticHeapDrain<'_, T, N> {
    StaticHeapDrain {
      iter: self.data.drain_iter(..),
    }
  }

  /// Drops all items from the StaticHeap.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let mut heap = StaticHeap::from(staticvec![1, 3]);
  /// assert!(!heap.is_empty());
  /// heap.clear();
  /// assert!(heap.is_empty());
  /// ```
  #[inline(always)]
  pub fn clear(&mut self) {
    self.drain();
  }
}
