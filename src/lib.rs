#![no_std]
#![allow(incomplete_features)]
#![feature(doc_cfg)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(maybe_uninit_extra)]
#![feature(exact_size_is_empty)]
#![feature(trusted_len)]
#![feature(slice_partition_dedup)]
#![feature(read_initializer)]

//Literally just for stable-sort.
#[cfg(any(feature = "std", rustdoc))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(rustdoc)]
use alloc::vec::Vec;

pub use crate::trait_impls::*;
use crate::utils::*;
use core::cmp::{Ord, PartialEq};
use core::mem::{self, MaybeUninit};
use core::ops::{Bound::Excluded, Bound::Included, Bound::Unbounded, RangeBounds};
use core::ptr;
use core::slice;

mod macros;
mod trait_impls;
#[doc(hidden)]
pub mod utils;

///A [Vec](alloc::vec::Vec)-like struct (mostly directly API-compatible where it can be)
///implemented with const generics around an array of fixed `N` capacity.
pub struct StaticVec<T, const N: usize> {
  length: usize,
  data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> StaticVec<T, { N }> {
  ///Returns a new StaticVec instance.
  #[inline(always)]
  pub fn new() -> Self {
    Self {
      //Sound because data is an array of MaybeUninit<T>, not an array of T.
      data: unsafe { MaybeUninit::uninit().assume_init() },
      length: 0,
    }
  }

  ///Returns a new StaticVec instance filled with the contents, if any, of a constant slice reference.
  ///If the slice has a length greater than the StaticVec's declared capacity,
  ///any contents after that point are ignored.
  ///Locally requires that `T` implements [Copy](core::marker::Copy) to avoid soundness issues.
  #[inline]
  pub fn new_from_slice(values: &[T]) -> Self
  where T: Copy {
    unsafe {
      let mut data: [MaybeUninit<T>; N] = MaybeUninit::uninit().assume_init();
      let length = values.len().min(N);
      values
        .as_ptr()
        .copy_to_nonoverlapping(data.as_mut_ptr() as *mut T, length);
      Self { data, length }
    }
  }

  ///Returns a new StaticVec instance filled with the contents, if any, of a mutable slice reference.
  ///If the slice has a length greater than the StaticVec's declared capacity,
  ///any contents after that point are ignored.
  ///Locally requires that `T` implements [Copy](core::marker::Copy) to avoid soundness issues.
  #[inline]
  pub fn new_from_mut_slice(values: &mut [T]) -> Self
  where T: Copy {
    Self::new_from_slice(values)
  }

  ///Returns a new StaticVec instance filled with the contents, if any, of an array.
  ///If the array has a length greater than the StaticVec's declared capacity,
  ///any contents after that point are ignored (and dropped)
  ///The `N2` parameter does not need to be provided explicitly, and can be inferred from the array itself.
  #[inline]
  pub fn new_from_array<const N2: usize>(mut values: [T; N2]) -> Self {
    // TODO: if N == N2, use transmute; this will allow the compiler to reuse
    // the array on the stack without copying anything
    let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
    let length = N.min(N2);

    // Copy values into data
    // Safety: length <= N2, so all the values exist. These values will be
    // forgotten at the end of this function.
    unsafe {
      values
        .as_ptr()
        .copy_to_nonoverlapping(data.as_mut_ptr() as *mut T, length);
    }

    // drop any extra values
    unsafe {
      ptr::drop_in_place(values.get_unchecked_mut(length..N2));
    }

    // Don't run destructors for the passed-in array
    mem::forget(values);

    Self { data, length }
  }

  /// Returns a new StaticVec instance filled with the return value of an initializer function.
  /// The length field of the newly created StaticVec will be equal to its capacity.
  ///
  /// Example usage:
  ///
  /// ```
  /// let mut i = 0;
  /// let v = StaticVec::<i32, 64>::filled_with(|| { i += 1; i });
  /// assert_eq!(v.len(), 64);
  /// assert_eq!(v[0], 1);
  /// assert_eq!(v[1], 2);
  /// assert_eq!(v[2], 3);
  /// assert_eq!(v[3], 4);
  /// ```
  #[inline]
  pub fn filled_with<F>(mut initializer: F) -> Self
  where F: FnMut() -> T {
    let mut res = Self::new();
    for _ in 0..N {
      // Safety: this is guaranteed to push N items into res
      unsafe { res.push_unchecked(initializer()) };
    }
    res
  }

  ///Returns the current length of the StaticVec.
  ///Just as for a normal [Vec](alloc::vec::Vec), this means the number of elements that
  ///have been added to it with `push`, `insert`, etc. except in the case
  ///that it has been set directly with the unsafe `set_len` function.
  #[inline(always)]
  pub const fn len(&self) -> usize {
    self.length
  }

  ///Returns the total capacity of the StaticVec.
  ///
  ///This is always equivalent to the generic `N` parameter it was declared with,
  ///which determines the fixed size of the backing array.
  #[inline(always)]
  pub const fn capacity(&self) -> usize {
    N
  }

  /// Returns the remaining capacity of the `StaticVec`.
  #[inline(always)]
  pub const fn remaining_capacity(&self) -> usize {
    self.capacity() - self.len()
  }

  /// Directly sets the `length` field of the StaticVec to `new_len`. Useful if you intend
  /// to write to it solely element-wise, but marked unsafe due to how it creates
  /// the potential for reading from unitialized memory later on.
  ///
  /// # Safety
  ///
  /// This function is only safe if it is associated with direct writes
  /// to the underlying storage, via [`as_mut_ptr`]. When increasing the
  /// length, additional items must be inserted into the array, and when
  /// decreasing it, they must be dropped in place. It is NEVER safe to set
  /// the length to greater than `N`.
  #[inline(always)]
  pub unsafe fn set_len(&mut self, new_len: usize) {
    debug_assert!(
      new_len <= N,
      "Attempted to unsafely set the length to greated than the capacity"
    );
    self.length = new_len;
  }

  /// Check if the `StaticVec` is empty; that is, if its length is 0.
  #[inline(always)]
  pub const fn is_empty(&self) -> bool {
    self.length == 0
  }

  ///Returns true if the current length of the StaticVec is equal to its capacity.
  #[inline(always)]
  pub const fn is_full(&self) -> bool {
    self.length == N
  }

  ///Returns a constant pointer to the first element of the StaticVec's internal array.
  #[inline(always)]
  pub fn as_ptr(&self) -> *const T {
    self.data.as_ptr() as *const T
  }

  ///Returns a mutable pointer to the first element of the StaticVec's internal array.
  #[inline(always)]
  pub fn as_mut_ptr(&mut self) -> *mut T {
    self.data.as_mut_ptr() as *mut T
  }

  ///Returns a constant reference to a slice of the StaticVec's inhabited area.
  #[inline(always)]
  pub fn as_slice(&self) -> &[T] {
    // Safety: elements from 0..length are guaranteed to have been initialized
    unsafe { slice::from_raw_parts(self.as_ptr(), self.length) }
  }

  ///Returns a mutable reference to a slice of the StaticVec's inhabited area.
  #[inline(always)]
  pub fn as_mut_slice(&mut self) -> &mut [T] {
    // Safety: elements from 0..length are guaranteed to have been initialized
    unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.length) }
  }

  /// Appends a value to the end of the StaticVec without asserting that
  /// its current length is less than `N`.
  ///
  /// # Safety
  ///
  /// This function is only safe to call if the current length is less than
  /// the capacity. It correctly updates the length, so it's suitable for use
  /// in loops that are guaranteed to not overflow the `StaticVec`.
  #[inline(always)]
  pub unsafe fn push_unchecked(&mut self, value: T) {
    debug_assert!(
      !self.is_full(),
      "Attempted to push_unchecked a full StaticVec"
    );
    self.data.get_unchecked_mut(self.length).write(value);
    self.length += 1;
  }

  /// Pops a value from the end of the StaticVec and returns it directly without asserting that
  /// the StaticVec's current length is greater than 0.
  ///
  /// # Safety
  ///
  /// This function is only safe to call if the current length is at least 1.
  #[inline(always)]
  pub unsafe fn pop_unchecked(&mut self) -> T {
    debug_assert!(
      !self.is_empty(),
      "Attempted to pop_unchecked an empty StaticVec"
    );
    self.length -= 1;
    self.data.get_unchecked(self.length).read()
  }

  /// Append an element to self. Panics if self.length == self.capactiy
  #[inline(always)]
  pub fn push(&mut self, value: T) {
    self.try_push(value).expect("StaticVec at capacity")
  }

  ///Pushes `value` to the StaticVec if its current length is less than its capacity,
  ///or returns an error indicating there's no remaining capacity otherwise.
  #[inline(always)]
  pub fn try_push(&mut self, value: T) -> Result<(), &'static str> {
    if self.length < N {
      // Safetly: self.length < N
      unsafe {
        self.push_unchecked(value);
      }
      Ok(())
    } else {
      Err("No space left!")
    }
  }

  ///Removes the value at the last position of the StaticVec and returns it in `Some` if
  ///the StaticVec has a current length greater than 0, and returns `None` otherwise.
  #[inline(always)]
  pub fn pop(&mut self) -> Option<T> {
    if self.length == 0 {
      None
    } else {
      // Safety: there is guaranteed to be an element in self
      Some(unsafe { self.pop_unchecked() })
    }
  }

  ///Asserts that `index` is less than the current length of the StaticVec,
  ///and if so removes the value at that position and returns it. Any values
  ///that exist in later positions are shifted to the left.
  #[inline]
  pub fn remove(&mut self, index: usize) -> T {
    assert!(index < self.length);
    unsafe {
      let p = self.as_mut_ptr().add(index);
      let res = p.read();
      p.offset(1).copy_to(p, self.length - index - 1);
      self.length -= 1;
      res
    }
  }

  ///Removes the first instance of `item` from the StaticVec if the item exists.
  #[inline(always)]
  pub fn remove_item(&mut self, item: &T) -> Option<T>
  where T: PartialEq {
    //Adapted this from normal Vec's implementation.
    if let Some(pos) = self.iter().position(|x| *x == *item) {
      Some(self.remove(pos))
    } else {
      None
    }
  }

  ///Returns `None` if `index` is greater than or equal to the current length of the StaticVec.
  ///Otherwise, removes the value at that position and returns it in `Some`, and then
  ///moves the last value in the StaticVec into the empty slot.
  #[inline(always)]
  pub fn swap_pop(&mut self, index: usize) -> Option<T> {
    if index < self.length {
      unsafe {
        let last_value = self.data.get_unchecked(self.length - 1).read();
        self.length -= 1;
        Some(self.as_mut_ptr().add(index).replace(last_value))
      }
    } else {
      None
    }
  }

  ///Asserts that `index` is less than the current length of the StaticVec,
  ///and if so removes the value at that position and returns it, and then
  ///moves the last value in the StaticVec into the empty slot.
  #[inline(always)]
  pub fn swap_remove(&mut self, index: usize) -> T {
    assert!(index < self.length);
    unsafe {
      let last_value = self.data.get_unchecked(self.length - 1).read();
      self.length -= 1;
      self.as_mut_ptr().add(index).replace(last_value)
    }
  }

  ///Asserts that the current length of the StaticVec is less than `N` and that
  ///`index` is less than the length, and if so inserts `value` at that position.
  ///Any values that exist in positions after `index` are shifted to the right.
  #[inline]
  pub fn insert(&mut self, index: usize, value: T) {
    assert!(self.length < N && index <= self.length);
    unsafe {
      let p = self.as_mut_ptr().add(index);
      p.copy_to(p.offset(1), self.length - index);
      p.write(value);
      self.length += 1;
    }
  }

  ///Inserts `value` at `index` if the current length of the StaticVec is less than `N` and `index`
  ///is less than the length, or returns a error stating one of the two is not the case otherwise.
  ///Any values that exist in positions after `index` are shifted to the right.
  #[inline]
  pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), &'static str> {
    if self.length < N && index <= self.length {
      unsafe {
        let p = self.as_mut_ptr().add(index);
        p.copy_to(p.offset(1), self.length - index);
        p.write(value);
        self.length += 1;
        Ok(())
      }
    } else {
      Err("One of `self.length < N` or `index <= self.length` is false!")
    }
  }

  ///Removes all contents from the StaticVec and sets its length back to 0.
  #[inline(always)]
  pub fn clear(&mut self) {
    unsafe { ptr::drop_in_place(self.as_mut_slice()) }
    self.length = 0;
  }

  ///Returns a separate, stable-sorted StaticVec of the contents of the
  ///StaticVec's inhabited area without modifying the original data.
  ///Locally requires that `T` implements [Copy](core::marker::Copy) to avoid soundness issues,
  ///and [Ord](core::cmp::Ord) to make the sorting possible.
  #[cfg(feature = "std")]
  #[inline]
  pub fn sorted(&self) -> Self
  where T: Copy + Ord {
    let mut res = Self::new_from_slice(&self);
    res.sort();
    res
  }

  ///Returns a separate, unstable-sorted StaticVec of the contents of the
  ///StaticVec's inhabited area without modifying the original data.
  ///Locally requires that `T` implements [Copy](core::marker::Copy) to avoid soundness issues,
  ///and [Ord](core::cmp::Ord) to make the sorting possible.
  #[inline]
  pub fn sorted_unstable(&self) -> Self
  where T: Copy + Ord {
    let mut res = Self::new_from_slice(&self);
    res.sort_unstable();
    res
  }

  ///Returns a separate, reversed StaticVec of the contents of the StaticVec's
  ///inhabited area without modifying the original data.
  ///Locally requires that `T` implements [Copy](core::marker::Copy) to avoid soundness issues.
  #[inline]
  pub fn reversed(&self) -> Self
  where T: Copy {
    let mut res = Self::new();
    res.length = self.length;
    unsafe {
      reverse_copy(
        self.as_ptr(),
        self.as_ptr().add(self.length),
        res.as_mut_ptr(),
      );
    }
    res
  }

  ///Copies and appends all elements, if any, of a slice to the StaticVec.
  ///If the slice has a length greater than the StaticVec's declared capacity,
  ///any contents after that point are ignored.
  ///Unlike the implementation of this function for [Vec](alloc::vec::Vec), no iterator is used,
  ///just a single pointer-copy call.
  ///Locally requires that `T` implements [Copy](core::marker::Copy) to avoid soundness issues.
  #[inline(always)]
  pub fn extend_from_slice(&mut self, other: &[T])
  where T: Copy {
    let added_length = (self.length + other.len()).min(N - self.length);
    unsafe {
      other
        .as_ptr()
        .copy_to_nonoverlapping(self.as_mut_ptr().add(self.length), added_length);
    }
    self.length += added_length;
  }

  ///Copies and appends all elements, if any, of a slice to the StaticVec if the
  ///StaticVec's remaining capacity is greater than the length of the slice, or returns
  ///an error indicating that's not the case otherwise.
  #[inline(always)]
  pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), &'static str>
  where T: Copy {
    if self.remaining_capacity() < other.len() {
      Err("No space left!")
    } else {
      self.extend_from_slice(other);
      Ok(())
    }
  }

  ///Removes the specified range of elements from the StaticVec and returns them in a new one.
  #[inline]
  pub fn drain<R>(&mut self, range: R) -> Self
  //No Copy bounds here because the original StaticVec gives up all access to the values in question.
  where R: RangeBounds<usize> {
    //Borrowed this part from normal Vec's implementation.
    let start = match range.start_bound() {
      Included(&idx) => idx,
      Excluded(&idx) => idx + 1,
      Unbounded => 0,
    };
    let end = match range.end_bound() {
      Included(&idx) => idx + 1,
      Excluded(&idx) => idx,
      Unbounded => self.length,
    };
    assert!(start <= end && end <= self.length);
    let mut res = Self::new();
    res.length = end - start;
    unsafe {
      self
        .as_ptr()
        .add(start)
        .copy_to_nonoverlapping(res.as_mut_ptr(), res.length);
      self
        .as_ptr()
        .add(end)
        .copy_to(self.as_mut_ptr().add(start), self.length - end);
    }
    self.length -= res.length;
    res
  }

  ///Removes all elements in the StaticVec for which `filter` returns true and
  ///returns them in a new one.
  #[inline]
  pub fn drain_filter<F>(&mut self, mut filter: F) -> Self
  where F: FnMut(&mut T) -> bool {
    let mut res = Self::new();
    let old_length = self.length;
    self.length = 0;
    unsafe {
      for i in 0..old_length {
        let val = self.as_mut_ptr().add(i);
        if filter(&mut *val) {
          res.data.get_unchecked_mut(res.length).write(val.read());
          res.length += 1;
        } else if res.length > 0 {
          self
            .as_ptr()
            .add(i)
            .copy_to_nonoverlapping(self.as_mut_ptr().add(i - res.length), 1);
        }
      }
    }
    self.length = old_length - res.length;
    res
  }

  ///Removes all elements in the StaticVec for which `filter` returns false.
  #[inline(always)]
  pub fn retain<F>(&mut self, mut filter: F)
  where F: FnMut(&T) -> bool {
    self.drain_filter(|val| !filter(val));
  }

  ///Shortens the StaticVec, keeping the first `length` elements and dropping the rest.
  ///Does nothing if `length` is greater than or equal to the current length of the StaticVec.
  #[inline(always)]
  pub fn truncate(&mut self, length: usize) {
    if length < self.length {
      self.length = length;
      // Safety: self.length has been updated, so these elements are no longer
      // in the array. length is < self.length, so get_unchecked_mut is safe.
      unsafe {
        ptr::drop_in_place(self.get_unchecked_mut(length..));
      }
    }
  }

  ///Splits the StaticVec into two at the given index.
  ///The original StaticVec will contain elements `0..at`,
  ///and the new one will contain elements `at..length`.
  #[inline]
  pub fn split_off(&mut self, at: usize) -> Self {
    assert!(at <= self.length);
    let split_length = self.length - at;
    let mut split = Self::new();
    unsafe {
      self.length = at;
      split.length = split_length;
      self
        .as_ptr()
        .add(at)
        .copy_to_nonoverlapping(split.as_mut_ptr(), split_length);
    }
    split
  }

  ///Removes all but the first of consecutive elements in the StaticVec satisfying a given equality relation.
  #[inline(always)]
  pub fn dedup_by<F>(&mut self, same_bucket: F)
  where F: FnMut(&mut T, &mut T) -> bool {
    //Exactly the same as Vec's version.
    let len = {
      let (dedup, _) = self.as_mut_slice().partition_dedup_by(same_bucket);
      dedup.len()
    };
    self.truncate(len);
  }

  ///Removes consecutive repeated elements in the StaticVec according to the
  ///locally required [PartialEq](core::cmp::PartialEq) trait implementation for `T`.
  #[inline(always)]
  pub fn dedup(&mut self)
  where T: PartialEq {
    //Exactly the same as Vec's version.
    self.dedup_by(|a, b| a == b)
  }

  ///Removes all but the first of consecutive elements in the StaticVec that
  ///resolve to the same key.
  #[inline(always)]
  pub fn dedup_by_key<F, K>(&mut self, mut key: F)
  where
    F: FnMut(&mut T) -> K,
    K: PartialEq<K>, {
    //Exactly the same as Vec's version.
    self.dedup_by(|a, b| key(a) == key(b))
  }
}
