#![allow(missing_docs)]
#![feature(const_generics)]
#![feature(trusted_len)]
#![feature(exact_size_is_empty)]

use core::fmt;
use core::iter::{FromIterator, FusedIterator, TrustedLen};
use core::mem::{swap, ManuallyDrop};
use core::ops::{Deref, DerefMut};
use core::ptr;

use staticvec::StaticVec;

/// A priority queue implemented with a binary heap.
///
/// This will be a max-heap.
///
/// It is a logic error for an item to be modified in such a way that the
/// item's ordering relative to any other item, as determined by the `Ord`
/// trait, changes while it is in the heap. This is normally only possible
/// through `Cell`, `RefCell`, global state, I/O, or unsafe code.
///
/// # Examples
///
/// ```
/// use std::collections::BinaryHeap;
///
/// // Type inference lets us omit an explicit type signature (which
/// // would be `BinaryHeap<i32>` in this example).
/// let mut heap = BinaryHeap::new();
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
///     println!("{}", x);
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
/// Either `std::cmp::Reverse` or a custom `Ord` implementation can be used to
/// make `BinaryHeap` a min-heap. This makes `heap.pop()` return the smallest
/// value instead of the greatest one.
///
/// ```
/// use std::collections::BinaryHeap;
/// use std::cmp::Reverse;
///
/// let mut heap = BinaryHeap::new();
///
/// // Wrap values in `Reverse`
/// heap.push(Reverse(1));
/// heap.push(Reverse(5));
/// heap.push(Reverse(2));
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

pub struct BinaryHeap<T, const K: usize> {
    data: StaticVec<T, K>,
}

/// Structure wrapping a mutable reference to the greatest item on a
/// `BinaryHeap`.
///
/// This `struct` is created by the [`peek_mut`] method on [`BinaryHeap`]. See
/// its documentation for more.
///
/// [`peek_mut`]: struct.BinaryHeap.html#method.peek_mut
/// [`BinaryHeap`]: struct.BinaryHeap.html

pub struct PeekMut<'a, T: 'a + Ord, const K: usize> {
    heap: &'a mut BinaryHeap<T, K>,
    sift: bool,
}

impl<T: Ord + fmt::Debug, const K: usize> fmt::Debug for PeekMut<'_, T, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PeekMut").field(&self.heap.data[0]).finish()
    }
}

impl<T: Ord, const K: usize> Drop for PeekMut<'_, T, K> {
    fn drop(&mut self) {
        if self.sift {
            self.heap.sift_down(0);
        }
    }
}

impl<T: Ord, const K: usize> Deref for PeekMut<'_, T, K> {
    type Target = T;
    fn deref(&self) -> &T {
        debug_assert!(!self.heap.is_empty());
        // SAFE: PeekMut is only instantiated for non-empty heaps
        unsafe { self.heap.data.get_unchecked(0) }
    }
}

impl<T: Ord, const K: usize> DerefMut for PeekMut<'_, T, K> {
    fn deref_mut(&mut self) -> &mut T {
        debug_assert!(!self.heap.is_empty());
        // SAFE: PeekMut is only instantiated for non-empty heaps
        unsafe { self.heap.data.get_unchecked_mut(0) }
    }
}

impl<'a, T: Ord, const K: usize> PeekMut<'a, T, K> {
    /// Removes the peeked value from the heap and returns it.

    pub fn pop(mut this: PeekMut<'a, T, K>) -> T {
        let value = this.heap.pop().unwrap();
        this.sift = false;
        value
    }
}

impl<T: Clone, const K: usize> Clone for BinaryHeap<T, K> {
    fn clone(&self) -> Self {
        BinaryHeap {
            data: self.data.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.data.clone_from(&source.data);
    }
}

impl<T: Ord, const K: usize> Default for BinaryHeap<T, K> {
    /// Creates an empty `BinaryHeap<T>`.
    #[inline]
    fn default() -> BinaryHeap<T, K> {
        BinaryHeap::new()
    }
}

impl<T: fmt::Debug, const K: usize> fmt::Debug for BinaryHeap<T, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Ord, const K: usize> BinaryHeap<T, K> {
    /// Creates an empty `BinaryHeap` as a max-heap.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::new();
    /// heap.push(4);
    /// ```

    pub fn new() -> BinaryHeap<T, K> {
        BinaryHeap {
            data: StaticVec::new(),
        }
    }

    /// Returns a mutable reference to the greatest item in the binary heap, or
    /// `None` if it is empty.
    ///
    /// Note: If the `PeekMut` value is leaked, the heap may be in an
    /// inconsistent state.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::new();
    /// assert!(heap.peek_mut().is_none());
    ///
    /// heap.push(1);
    /// heap.push(5);
    /// heap.push(2);
    /// {
    ///     let mut val = heap.peek_mut().unwrap();
    ///     *val = 0;
    /// }
    /// assert_eq!(heap.peek(), Some(&2));
    /// ```
    ///
    /// # Time complexity
    ///
    /// Cost is O(1) in the worst case.

    pub fn peek_mut(&mut self) -> Option<PeekMut<'_, T, K>> {
        if self.is_empty() {
            None
        } else {
            Some(PeekMut {
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
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::from(vec![1, 3]);
    ///
    /// assert_eq!(heap.pop(), Some(3));
    /// assert_eq!(heap.pop(), Some(1));
    /// assert_eq!(heap.pop(), None);
    /// ```
    ///
    /// # Time complexity
    ///
    /// The worst case cost of `pop` on a heap containing *n* elements is O(log
    /// n).

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|mut item| {
            if !self.is_empty() {
                swap(&mut item, &mut self.data[0]);
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
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::new();
    /// heap.push(3);
    /// heap.push(5);
    /// heap.push(1);
    ///
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

    pub fn push(&mut self, item: T) {
        let old_len = self.len();
        self.data.push(item);
        self.sift_up(0, old_len);
    }

    /// Consumes the `BinaryHeap` and returns a vector in sorted
    /// (ascending) order.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    ///
    /// let mut heap = BinaryHeap::from(vec![1, 2, 4, 5, 7]);
    /// heap.push(6);
    /// heap.push(3);
    ///
    /// let vec = heap.into_sorted_vec();
    /// assert_eq!(vec, [1, 2, 3, 4, 5, 6, 7]);
    /// ```

    pub fn into_sorted_vec(mut self) -> StaticVec<T, K> {
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
    // The `Hole` type is used to represent this, and make sure
    // the hole is filled back at the end of its scope, even on panic.
    // Using a hole reduces the constant factor compared to using swaps,
    // which involves twice as many moves.
    fn sift_up(&mut self, start: usize, pos: usize) -> usize {
        unsafe {
            // Take out the value at `pos` and create a hole.
            let mut hole = Hole::new(&mut self.data, pos);

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

    /// Take an element at `pos` and move it down the heap,
    /// while its children are larger.
    fn sift_down_range(&mut self, pos: usize, end: usize) {
        unsafe {
            let mut hole = Hole::new(&mut self.data, pos);
            let mut child = 2 * pos + 1;
            while child < end {
                let right = child + 1;
                // compare with the greater of the two children
                if right < end && !(hole.get(child) > hole.get(right)) {
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

    fn sift_down(&mut self, pos: usize) {
        let len = self.len();
        self.sift_down_range(pos, len);
    }

    /// Take an element at `pos` and move it all the way down the heap,
    /// then sift it up to its position.
    ///
    /// Note: This is faster when the element is known to be large / should
    /// be closer to the bottom.
    fn sift_down_to_bottom(&mut self, mut pos: usize) {
        let end = self.len();
        let start = pos;
        unsafe {
            let mut hole = Hole::new(&mut self.data, pos);
            let mut child = 2 * pos + 1;
            while child < end {
                let right = child + 1;
                // compare with the greater of the two children
                if right < end && !(hole.get(child) > hole.get(right)) {
                    child = right;
                }
                hole.move_to(child);
                child = 2 * hole.pos() + 1;
            }
            pos = hole.pos;
        }
        self.sift_up(start, pos);
    }

    fn rebuild(&mut self) {
        let mut n = self.len() / 2;
        while n > 0 {
            n -= 1;
            self.sift_down(n);
        }
    }

    /// Returns an iterator which retrieves elements in heap order.
    /// The retrieved elements are removed from the original heap.
    /// The remaining elements will be removed on drop in heap order.
    ///
    /// Note:
    /// * `.drain_sorted()` is O(n lg n); much slower than `.drain()`.
    ///   You should use the latter for most cases.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// #![feature(binary_heap_drain_sorted)]
    /// use std::collections::BinaryHeap;
    ///
    /// let mut heap = BinaryHeap::from(vec![1, 2, 3, 4, 5]);
    /// assert_eq!(heap.len(), 5);
    ///
    /// drop(heap.drain_sorted()); // removes all elements in heap order
    /// assert_eq!(heap.len(), 0);
    /// ```
    #[inline]

    pub fn drain_sorted(&mut self) -> DrainSorted<'_, T, K> {
        DrainSorted { inner: self }
    }
}

impl<T, const K: usize> BinaryHeap<T, K> {
    /// Returns an iterator visiting all values in the underlying vector, in
    /// arbitrary order.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let heap = BinaryHeap::from(vec![1, 2, 3, 4]);
    ///
    /// // Print 1, 2, 3, 4 in arbitrary order
    /// for x in heap.iter() {
    ///     println!("{}", x);
    /// }
    /// ```

    pub fn iter(&self) -> staticvec::StaticVecIterConst<'_, T, K> {
        self.data.iter()
    }

    /// Returns an iterator which retrieves elements in heap order.
    /// This method consumes the original heap.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// #![feature(binary_heap_into_iter_sorted)]
    /// use std::collections::BinaryHeap;
    /// let heap = BinaryHeap::from(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(heap.into_iter_sorted().take(2).collect::<Vec<_>>(), vec![5, 4]);
    /// ```

    pub fn into_iter_sorted(self) -> IntoIterSorted<T, K> {
        IntoIterSorted { inner: self }
    }

    /// Returns the greatest item in the binary heap, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::new();
    /// assert_eq!(heap.peek(), None);
    ///
    /// heap.push(1);
    /// heap.push(5);
    /// heap.push(2);
    /// assert_eq!(heap.peek(), Some(&5));
    ///
    /// ```
    ///
    /// # Time complexity
    ///
    /// Cost is O(1) in the worst case.

    pub fn peek(&self) -> Option<&T> {
        self.data.get(0)
    }

    /// Returns the number of elements the binary heap can hold without reallocating.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::with_capacity(100);
    /// assert!(heap.capacity() >= 100);
    /// heap.push(4);
    /// ```

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Consumes the `BinaryHeap` and returns the underlying vector
    /// in arbitrary order.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let heap = BinaryHeap::from(vec![1, 2, 3, 4, 5, 6, 7]);
    /// let vec = heap.into_vec();
    ///
    /// // Will print in some order
    /// for x in vec {
    ///     println!("{}", x);
    /// }
    /// ```

    pub fn into_vec(self) -> StaticVec<T, K> {
        self.into()
    }

    /// Returns the length of the binary heap.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let heap = BinaryHeap::from(vec![1, 3]);
    ///
    /// assert_eq!(heap.len(), 2);
    /// ```

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Checks if the binary heap is empty.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::new();
    ///
    /// assert!(heap.is_empty());
    ///
    /// heap.push(3);
    /// heap.push(5);
    /// heap.push(1);
    ///
    /// assert!(!heap.is_empty());
    /// ```

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the binary heap, returning an iterator over the removed elements.
    ///
    /// The elements are removed in arbitrary order.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::from(vec![1, 3]);
    ///
    /// assert!(!heap.is_empty());
    ///
    /// for x in heap.drain() {
    ///     println!("{}", x);
    /// }
    ///
    /// assert!(heap.is_empty());
    /// ```

    #[inline]
    pub fn drain(&mut self) -> Drain<'_, T, K> {
        Drain {
            iter: self.data.drain_iter(..),
        }
    }

    /// Drops all items from the binary heap.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let mut heap = BinaryHeap::from(vec![1, 3]);
    ///
    /// assert!(!heap.is_empty());
    ///
    /// heap.clear();
    ///
    /// assert!(heap.is_empty());
    /// ```

    pub fn clear(&mut self) {
        self.drain();
    }
}

/// Hole represents a hole in a slice i.e., an index without valid value
/// (because it was moved from or duplicated).
/// In drop, `Hole` will restore the slice by filling the hole
/// position with the value that was originally removed.
struct Hole<'a, T: 'a> {
    data: &'a mut [T],
    elt: ManuallyDrop<T>,
    pos: usize,
}

impl<'a, T> Hole<'a, T> {
    /// Create a new `Hole` at index `pos`.
    ///
    /// Unsafe because pos must be within the data slice.
    #[inline]
    unsafe fn new(data: &'a mut [T], pos: usize) -> Self {
        debug_assert!(pos < data.len());
        // SAFE: pos should be inside the slice
        let elt = ptr::read(data.get_unchecked(pos));
        Hole {
            data,
            elt: ManuallyDrop::new(elt),
            pos,
        }
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }

    /// Returns a reference to the element removed.
    #[inline]
    fn element(&self) -> &T {
        &self.elt
    }

    /// Returns a reference to the element at `index`.
    ///
    /// Unsafe because index must be within the data slice and not equal to pos.
    #[inline]
    unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        self.data.get_unchecked(index)
    }

    /// Move hole to new location
    ///
    /// Unsafe because index must be within the data slice and not equal to pos.
    #[inline]
    unsafe fn move_to(&mut self, index: usize) {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        let index_ptr: *const _ = self.data.get_unchecked(index);
        let hole_ptr = self.data.get_unchecked_mut(self.pos);
        ptr::copy_nonoverlapping(index_ptr, hole_ptr, 1);
        self.pos = index;
    }
}

impl<T> Drop for Hole<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // fill the hole again
        unsafe {
            let pos = self.pos;
            ptr::copy_nonoverlapping(&*self.elt, self.data.get_unchecked_mut(pos), 1);
        }
    }
}

/// An iterator over the elements of a `BinaryHeap`.
///
/// This `struct` is created by the [`iter`] method on [`BinaryHeap`]. See its
/// documentation for more.
///
/// [`iter`]: struct.BinaryHeap.html#method.iter
/// [`BinaryHeap`]: struct.BinaryHeap.html

pub struct Iter<'a, T: 'a, const K: usize> {
    iter: staticvec::StaticVecIterConst<'a, T, K>,
}

impl<T: fmt::Debug, const K: usize> fmt::Debug for Iter<'_, T, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Iter").field(&self.iter.as_slice()).finish()
    }
}

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`

impl<T, const K: usize> Clone for Iter<'_, T, K> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T, const K: usize> Iterator for Iter<'a, T, K> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<&'a T> {
        self.iter.last()
    }
}

impl<'a, T, const K: usize> DoubleEndedIterator for Iter<'a, T, K> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<T, const K: usize> ExactSizeIterator for Iter<'_, T, K> {
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<T, const K: usize> FusedIterator for Iter<'_, T, K> {}

/// An owning iterator over the elements of a `BinaryHeap`.
///
/// This `struct` is created by the [`into_iter`] method on [`BinaryHeap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: struct.BinaryHeap.html#method.into_iter
/// [`BinaryHeap`]: struct.BinaryHeap.html

//FIXME: ???
//#[derive(Clone)]
pub struct IntoIter<T, const K: usize> {
    iter: staticvec::StaticVecIntoIter<T, K>,
}

impl<T: fmt::Debug, const K: usize> fmt::Debug for IntoIter<T, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter")
            .field(&self.iter.as_slice())
            .finish()
    }
}

impl<T, const K: usize> Iterator for IntoIter<T, K> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, const K: usize> DoubleEndedIterator for IntoIter<T, K> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T, const K: usize> ExactSizeIterator for IntoIter<T, K> {
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<T, const K: usize> FusedIterator for IntoIter<T, K> {}

#[derive(Clone, Debug)]
pub struct IntoIterSorted<T, const K: usize> {
    inner: BinaryHeap<T, K>,
}

impl<T: Ord, const K: usize> Iterator for IntoIterSorted<T, K> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.inner.pop()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.inner.len();
        (exact, Some(exact))
    }
}

impl<T: Ord, const K: usize> ExactSizeIterator for IntoIterSorted<T, K> {}

impl<T: Ord, const K: usize> FusedIterator for IntoIterSorted<T, K> {}

unsafe impl<T: Ord, const K: usize> TrustedLen for IntoIterSorted<T, K> {}

/// A draining iterator over the elements of a `BinaryHeap`.
///
/// This `struct` is created by the [`drain`] method on [`BinaryHeap`]. See its
/// documentation for more.
///
/// [`drain`]: struct.BinaryHeap.html#method.drain
/// [`BinaryHeap`]: struct.BinaryHeap.html

#[derive(Debug)]
pub struct Drain<'a, T: 'a, const K: usize> {
    iter: staticvec::StaticVecDrain<'a, T, K>,
}

impl<T, const K: usize> Iterator for Drain<'_, T, K> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, const K: usize> DoubleEndedIterator for Drain<'_, T, K> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T, const K: usize> ExactSizeIterator for Drain<'_, T, K> {
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<T, const K: usize> FusedIterator for Drain<'_, T, K> {}

/// A draining iterator over the elements of a `BinaryHeap`.
///
/// This `struct` is created by the [`drain_sorted`] method on [`BinaryHeap`]. See its
/// documentation for more.
///
/// [`drain_sorted`]: struct.BinaryHeap.html#method.drain_sorted
/// [`BinaryHeap`]: struct.BinaryHeap.html

#[derive(Debug)]
pub struct DrainSorted<'a, T: Ord, const K: usize> {
    inner: &'a mut BinaryHeap<T, K>,
}

impl<'a, T: Ord, const K: usize> Drop for DrainSorted<'a, T, K> {
    /// Removes heap elements in heap order.
    fn drop(&mut self) {
        while let Some(_) = self.inner.pop() {}
    }
}

impl<T: Ord, const K: usize> Iterator for DrainSorted<'_, T, K> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.inner.pop()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.inner.len();
        (exact, Some(exact))
    }
}

impl<T: Ord, const K: usize> ExactSizeIterator for DrainSorted<'_, T, K> {}

impl<T: Ord, const K: usize> FusedIterator for DrainSorted<'_, T, K> {}

unsafe impl<T: Ord, const K: usize> TrustedLen for DrainSorted<'_, T, K> {}

impl<T: Ord, const K: usize> From<StaticVec<T, K>> for BinaryHeap<T, K> {
    /// Converts a `Vec<T>` into a `BinaryHeap<T>`.
    ///
    /// This conversion happens in-place, and has `O(n)` time complexity.
    fn from(vec: StaticVec<T, K>) -> BinaryHeap<T, K> {
        let mut heap = BinaryHeap { data: vec };
        heap.rebuild();
        heap
    }
}

impl<T, const K: usize> From<BinaryHeap<T, K>> for StaticVec<T, K> {
    fn from(heap: BinaryHeap<T, K>) -> StaticVec<T, K> {
        heap.data
    }
}

impl<T: Ord, const K: usize> FromIterator<T> for BinaryHeap<T, K> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> BinaryHeap<T, K> {
        BinaryHeap::from(iter.into_iter().collect::<StaticVec<_, K>>())
    }
}

impl<T, const K: usize> IntoIterator for BinaryHeap<T, K> {
    type Item = T;
    type IntoIter = IntoIter<T, K>;

    /// Creates a consuming iterator, that is, one that moves each value out of
    /// the binary heap in arbitrary order. The binary heap cannot be used
    /// after calling this.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BinaryHeap;
    /// let heap = BinaryHeap::from(vec![1, 2, 3, 4]);
    ///
    /// // Print 1, 2, 3, 4 in arbitrary order
    /// for x in heap.into_iter() {
    ///     // x has type i32, not &i32
    ///     println!("{}", x);
    /// }
    /// ```
    fn into_iter(self) -> IntoIter<T, K> {
        IntoIter {
            iter: self.data.into_iter(),
        }
    }
}

impl<'a, T, const K: usize> IntoIterator for &'a BinaryHeap<T, K> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, K>;

    fn into_iter(self) -> Iter<'a, T, K> {
        Iter { iter: self.iter() }
    }
}