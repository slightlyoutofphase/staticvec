#![no_std]
#![allow(clippy::doc_markdown)]
#![allow(clippy::inline_always)]
#![allow(incomplete_features)]
#![feature(const_compare_raw_pointers)]
#![feature(const_fn)]
#![feature(const_fn_union)]
#![feature(const_generics)]
#![feature(const_if_match)]
#![feature(const_loop)]
#![feature(const_mut_refs)]
#![feature(const_panic)]
#![feature(const_raw_ptr_deref)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(core_intrinsics)]
#![feature(doc_cfg)]
#![feature(exact_size_is_empty)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_ref)]
#![feature(maybe_uninit_uninit_array)]
#![feature(read_initializer)]
#![feature(slice_partition_dedup)]
#![feature(specialization)]
#![feature(trusted_len)]
#![feature(untagged_unions)]

pub use crate::errors::{CapacityError, PushCapacityError};
pub use crate::iterators::*;
pub use crate::trait_impls::*;
use crate::utils::{slice_from_raw_parts, slice_from_raw_parts_mut, reverse_copy};
use core::cmp::{Ord, PartialEq};
use core::intrinsics;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{
  Add, Bound::Excluded, Bound::Included, Bound::Unbounded, Div, Mul, RangeBounds, Sub,
};
use core::ptr;

#[cfg(any(feature = "std", rustdoc))]
extern crate alloc;

#[cfg(any(feature = "std", rustdoc))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
extern crate std;

mod iterators;
#[macro_use]
mod macros;
#[doc(hidden)]
mod errors;
mod trait_impls;
#[doc(hidden)]
pub mod utils;

/// A [`Vec`](alloc::vec::Vec)-like struct (mostly directly API-compatible where it can be)
/// implemented with const generics around an array of fixed `N` capacity.
pub struct StaticVec<T, const N: usize> {
  // We create this field in an uninitialized state, and write to it element-wise as needed
  // via pointer methods. At no time should `assume_init` *ever* be called through it.
  data: MaybeUninit<[T; N]>,
  // The constant `N` parameter (and thus the total span of `data`) represent capacity for us,
  // while the field below represents, as its name suggests, the current length of a StaticVec
  // (that is, the current number of "live" elements) just as is the case for a regular `Vec`.
  length: usize,
}

impl<T, const N: usize> StaticVec<T, N> {
  /// Returns a new StaticVec instance.
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      data: Self::new_data_uninit(),
      length: 0,
    }
  }

  /// Returns a new StaticVec instance filled with the contents, if any, of a slice reference,
  /// which can be either `&mut` or `&` as if it is `&mut` it will implicitly coerce to `&`.
  /// If the slice has a length greater than the StaticVec's declared capacity,
  /// any contents after that point are ignored.
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to avoid soundness issues.
  #[inline]
  pub fn new_from_slice(values: &[T]) -> Self
  where T: Copy {
    let length = values.len().min(N);
    Self {
      data: {
        let mut data = Self::new_data_uninit();
        unsafe {
          values
            .as_ptr()
            .copy_to_nonoverlapping(Self::first_ptr_mut(&mut data), length);
          data
        }
      },
      length,
    }
  }

  /// Returns a new StaticVec instance filled with the contents, if any, of an array.
  /// If the array has a length greater than the StaticVec's declared capacity,
  /// any contents after that point are ignored.
  ///
  /// The `N2` parameter does not need to be provided explicitly, and can be inferred from the array
  /// itself.
  ///
  /// This function does *not* leak memory, as any ignored extra elements in the source
  /// array are explicitly dropped with [`drop_in_place`](core::ptr::drop_in_place) after it is
  /// first wrapped in an instance of [`MaybeUninit`](core::mem::MaybeUninit) to inhibit the
  /// automatic calling of any destructors its contents may have.
  ///
  /// Example usage:
  /// ```
  /// // Same input length as the declared capacity:
  /// let v = StaticVec::<i32, 3>::new_from_array([1, 2, 3]);
  /// assert_eq!(v, [1, 2, 3]);
  /// // Truncated to fit the declared capacity:
  /// let v2 = StaticVec::<i32, 3>::new_from_array([1, 2, 3, 4, 5, 6]);
  /// assert_eq!(v2, [1, 2, 3]);
  /// ```
  /// Note that StaticVec also implements [`From`](core::convert::From) for both slices
  /// and static arrays, which may prove more ergonomic in some cases as it allows
  /// for a greater degree of type inference:
  /// ```
  /// // The StaticVec on the next line is inferred to be of type `StaticVec<&'static str, 4>`.
  /// let v = StaticVec::from(["A", "B", "C", "D"]);
  /// ```
  #[inline]
  pub fn new_from_array<const N2: usize>(values: [T; N2]) -> Self {
    if N == N2 {
      Self::from(values)
    } else {
      Self {
        data: {
          unsafe {
            let mut data = Self::new_data_uninit();
            values
              .as_ptr()
              .copy_to_nonoverlapping(Self::first_ptr_mut(&mut data), N2.min(N));
            // Wrap the values in a MaybeUninit to inhibit their destructors (if any),
            // then manually drop any excess ones.
            let mut forgotten = MaybeUninit::new(values);
            ptr::drop_in_place(forgotten.get_mut().get_unchecked_mut(N2.min(N)..N2));
            data
          }
        },
        length: N2.min(N),
      }
    }
  }

  /// A version of [`new_from_array`](crate::StaticVec::new_from_array) specifically designed
  /// for use as a `const fn` constructor (although it can of course be used in non-const contexts
  /// as well.)
  ///
  /// Being `const` necessitates that this function can only accept arrays with a length
  /// exactly equal to the declared capacity of the resulting StaticVec, so if you do need
  /// flexibility with regards to input lengths it's recommended that you use
  /// [`new_from_array`](crate::StaticVec::new_from_array) or the [`From`](core::convert::From)
  /// implementations instead.
  ///
  /// Note that both forms of the [`staticvec!`] macro are implemented using
  /// [`new_from_const_array`](crate::StaticVec::new_from_const_array), so you may also prefer
  /// to use them instead of it directly.
  #[inline(always)]
  pub const fn new_from_const_array(values: [T; N]) -> Self {
    Self {
      data: MaybeUninit::new(values),
      length: N,
    }
  }

  /// Returns the current length of the StaticVec. Just as for a normal [`Vec`](alloc::vec::Vec),
  /// this means the number of elements that have been added to it with
  /// [`push`](crate::StaticVec::push), [`insert`](crate::StaticVec::insert), etc. except in the
  /// case that it has been set directly with the unsafe [`set_len`](crate::StaticVec::set_len)
  /// function.
  #[inline(always)]
  pub const fn len(&self) -> usize {
    self.length
  }

  /// Returns the total capacity of the StaticVec.
  /// This is always equivalent to the generic `N` parameter it was declared with,
  /// which determines the fixed size of the backing array.
  #[inline(always)]
  pub const fn capacity(&self) -> usize {
    N
  }

  /// Does the same thing as [`capacity`](crate::StaticVec::capacity), but as an associated
  /// function rather than a method.
  #[inline(always)]
  pub const fn cap() -> usize {
    N
  }

  /// Serves the same purpose as [`capacity`](crate::StaticVec::capacity), but as an associated
  /// constant rather than a method.
  pub const CAPACITY: usize = N;

  /// Returns the remaining capacity of the StaticVec.
  #[inline(always)]
  pub const fn remaining_capacity(&self) -> usize {
    N - self.length
  }

  /// Returns the total size of the inhabited part of the StaticVec (which may be zero if it has a
  /// length of zero or contains ZSTs) in bytes. Specifically, the return value of this function
  /// amounts to a calculation of `size_of::<T>() * self.length`.
  #[inline(always)]
  pub const fn size_in_bytes(&self) -> usize {
    intrinsics::size_of::<T>() * self.length
  }

  /// Directly sets the length field of the StaticVec to `new_len`. Useful if you intend
  /// to write to it solely element-wise, but marked unsafe due to how it creates
  /// the potential for reading from uninitialized memory later on.
  ///
  /// # Safety
  ///
  /// It is up to the caller to ensure that `new_len` is less than or equal to the StaticVec's
  /// constant `N` parameter, and that the range of elements covered by a length of `new_len` is
  /// actually initialized. Failure to do so will almost certainly result in undefined behavior.
  #[inline(always)]
  pub unsafe fn set_len(&mut self, new_len: usize) {
    // Most of the `unsafe` functions in this crate that are heavily used internally
    // have debug-build-only assertions where it's useful.
    debug_assert!(
      new_len <= N,
      "In `StaticVec::set_len`, provided length {} exceeds the maximum capacity of {}!",
      new_len,
      N
    );
    self.length = new_len;
  }

  /// Returns true if the current length of the StaticVec is 0.
  #[inline(always)]
  pub const fn is_empty(&self) -> bool {
    self.length == 0
  }

  /// Returns true if the current length of the StaticVec is greater than 0.
  #[inline(always)]
  pub const fn is_not_empty(&self) -> bool {
    self.length > 0
  }

  /// Returns true if the current length of the StaticVec is equal to its capacity.
  #[inline(always)]
  pub const fn is_full(&self) -> bool {
    self.length == N
  }

  /// Returns true if the current length of the StaticVec is less than its capacity.
  #[inline(always)]
  pub const fn is_not_full(&self) -> bool {
    self.length < N
  }

  /// Returns a constant pointer to the first element of the StaticVec's internal array.
  #[inline(always)]
  pub const fn as_ptr(&self) -> *const T {
    // Written like this so it can be `const fn`.
    &self.data as *const _ as *const T
  }

  /// Returns a mutable pointer to the first element of the StaticVec's internal array.
  #[inline(always)]
  pub const fn as_mut_ptr(&mut self) -> *mut T {
    // Written like this so it can be `const fn`.
    &mut self.data as *mut _ as *mut T
  }

  /// Returns a constant reference to a slice of the StaticVec's inhabited area.
  #[inline(always)]
  pub const fn as_slice(&self) -> &[T] {
    // Safety: `self.as_ptr()` is a pointer to an array for which the first `length`
    // elements are guaranteed to be initialized. Therefore this is a valid slice.
    slice_from_raw_parts(self.as_ptr(), self.length)
  }

  /// Returns a mutable reference to a slice of the StaticVec's inhabited area.
  #[inline(always)]
  pub const fn as_mut_slice(&mut self) -> &mut [T] {
    // Safety: See as_slice.
    slice_from_raw_parts_mut(self.as_mut_ptr(), self.length)
  }

  /// Returns a constant pointer to the element of the StaticVec at `index` without doing any
  /// checking to ensure that `index` is actually within any particular bounds. The return value of
  /// this function is equivalent to what would be returned from `as_ptr().add(index)`.
  ///
  /// # Safety
  ///
  /// It is up to the caller to ensure that `index` is within the appropriate bounds such that the
  /// function returns a pointer to a location that falls somewhere inside the full span of the
  /// StaticVec's backing array, and that if reading from the returned pointer, it has *already*
  /// been initialized properly.
  #[inline(always)]
  pub unsafe fn ptr_at_unchecked(&self, index: usize) -> *const T {
    // We check against `N` as opposed to `length` in our debug assertion here, as these
    // `_unchecked` versions of `ptr_at` and `mut_ptr_at` are primarily intended for
    // initialization-related purposes (and used extensively that way internally throughout the
    // crate.)
    debug_assert!(
      index <= N,
      "In `StaticVec::ptr_at_unchecked`, provided index {} must be within `0..={}`!",
      index,
      N
    );
    self.as_ptr().add(index)
  }

  /// Returns a mutable pointer to the element of the StaticVec at `index` without doing any
  /// checking to ensure that `index` is actually within any particular bounds. The return value of
  /// this function is equivalent to what would be returned from `as_mut_ptr().add(index)`.
  ///
  /// # Safety
  ///
  /// It is up to the caller to ensure that `index` is within the appropriate bounds such that the
  /// function returns a pointer to a location that falls somewhere inside the full span of the
  /// StaticVec's backing array.
  ///
  /// It is also the responsibility of the caller to ensure that the `length` field of the StaticVec
  /// is adjusted to properly reflect whatever range of elements this function may be used to
  /// initialize, and that if reading from the returned pointer, it has *already* been initialized
  /// properly.
  #[inline(always)]
  pub unsafe fn mut_ptr_at_unchecked(&mut self, index: usize) -> *mut T {
    // We check against `N` as opposed to `length` in our debug assertion here, as these
    // `_unchecked` versions of `ptr_at` and `mut_ptr_at` are primarily intended for
    // initialization-related purposes (and used extensively that way internally throughout the
    // crate.)
    debug_assert!(
      index <= N,
      "In `StaticVec::mut_ptr_at_unchecked`, provided index {} must be within `0..={}`!",
      index,
      N
    );
    self.as_mut_ptr().add(index)
  }

  /// Returns a constant pointer to the element of the StaticVec at `index` if `index`
  /// is within the range `0..self.length`, or panics if it is not. The return value of this
  /// function is equivalent to what would be returned from `as_ptr().add(index)`.
  #[inline(always)]
  pub fn ptr_at(&self, index: usize) -> *const T {
    assert!(
      index < self.length,
      "In `StaticVec::ptr_at`, provided index {} must be within `0..{}`!",
      index,
      self.length
    );
    unsafe { self.ptr_at_unchecked(index) }
  }

  /// Returns a mutable pointer to the element of the StaticVec at `index` if `index`
  /// is within the range `0..self.length`, or panics if it is not. The return value of this
  /// function is equivalent to what would be returned from `as_mut_ptr().add(index)`.
  #[inline(always)]
  pub fn mut_ptr_at(&mut self, index: usize) -> *mut T {
    assert!(
      index < self.length,
      "In `StaticVec::mut_ptr_at`, provided index {} must be within `0..{}`!",
      index,
      self.length
    );
    unsafe { self.mut_ptr_at_unchecked(index) }
  }

  /// Returns a constant reference to the element of the StaticVec at `index` without doing any
  /// checking to ensure that `index` is actually within any particular bounds.
  ///
  /// Note that unlike [`slice::get_unchecked`](https://doc.rust-lang.org/nightly/std/primitive.slice.html#method.get_unchecked),
  /// this method only supports accessing individual elements via `usize`; it cannot also produce
  /// subslices. To get a subslice without a bounds check, use
  /// `self.as_slice().get_unchecked(a..b)`.
  ///
  /// # Safety
  ///
  /// It is up to the caller to ensure that `index` is within the range `0..self.length`.
  #[inline(always)]
  pub unsafe fn get_unchecked(&self, index: usize) -> &T {
    // This function is used internally in places where `length` has been intentionally
    // temporarily set to zero, so we do our debug assertion against `N`.
    debug_assert!(
      index < N,
      "In `StaticVec::get_unchecked`, provided index {} must be within `0..{}`!",
      index,
      N
    );
    &*self.ptr_at_unchecked(index)
  }

  /// Returns a mutable reference to the element of the StaticVec at `index` without doing any
  /// checking to ensure that `index` is actually within any particular bounds.
  ///
  /// The same differences between this method and the slice method of the same name
  /// apply as do for [`get_unchecked`](crate::StaticVec::get_unchecked).
  ///
  /// # Safety
  ///
  /// It is up to the caller to ensure that `index` is within the range `0..self.length`.
  #[inline(always)]
  pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
    // This function is used internally in places where `length` has been intentionally
    // temporarily set to zero, so we do our debug assertion against `N`.
    debug_assert!(
      index < N,
      "In `StaticVec::get_unchecked_mut`, provided index {} must be within `0..{}`!",
      index,
      N
    );
    &mut *self.mut_ptr_at_unchecked(index)
  }

  /// Appends a value to the end of the StaticVec without asserting that
  /// its current length is less than `N`.
  ///
  /// # Safety
  ///
  /// It is up to the caller to ensure that the length of the StaticVec
  /// prior to using this function is less than `N`. Failure to do so will result
  /// in writing to an out-of-bounds memory region.
  #[inline(always)]
  pub unsafe fn push_unchecked(&mut self, value: T) {
    debug_assert!(
      self.is_not_full(),
      "`StaticVec::push_unchecked` was called through a StaticVec already at maximum capacity!"
    );
    let length = self.length;
    self.mut_ptr_at_unchecked(length).write(value);
    self.set_len(length + 1);
  }

  /// Pops a value from the end of the StaticVec and returns it directly without asserting that
  /// the StaticVec's current length is greater than 0.
  ///
  /// # Safety
  ///
  /// It is up to the caller to ensure that the StaticVec contains at least one
  /// element prior to using this function. Failure to do so will result in reading
  /// from uninitialized memory.
  #[inline(always)]
  pub unsafe fn pop_unchecked(&mut self) -> T {
    debug_assert!(
      self.is_not_empty(),
      "`StaticVec::pop_unchecked` was called through an empty StaticVec!"
    );
    let new_length = self.length - 1;
    self.set_len(new_length);
    self.ptr_at_unchecked(new_length).read()
  }

  /// Pushes `value` to the StaticVec if its current length is less than its capacity,
  /// or returns a [`PushCapacityError`](crate::errors::PushCapacityError) otherwise.
  #[inline(always)]
  pub fn try_push(&mut self, value: T) -> Result<(), PushCapacityError<T, N>> {
    if self.is_not_full() {
      unsafe {
        self.push_unchecked(value);
      };
      Ok(())
    } else {
      Err(PushCapacityError::new(value))
    }
  }

  /// Pushes a value to the end of the StaticVec. Panics if the collection is
  /// full; that is, if `self.len() == self.capacity()`.
  #[inline(always)]
  pub fn push(&mut self, value: T) {
    assert!(
      self.is_not_full(),
      "`StaticVec::push` was called through a StaticVec already at maximum capacity!"
    );
    unsafe { self.push_unchecked(value) };
  }

  /// Removes the value at the last position of the StaticVec and returns it in `Some` if
  /// the StaticVec has a current length greater than 0, and returns `None` otherwise.
  #[inline(always)]
  pub fn pop(&mut self) -> Option<T> {
    if self.is_empty() {
      None
    } else {
      Some(unsafe { self.pop_unchecked() })
    }
  }

  /// Returns a constant reference to the first element of the StaticVec in `Some` if the StaticVec
  /// is not empty, or `None` otherwise.
  #[inline(always)]
  pub fn first(&self) -> Option<&T> {
    if self.is_empty() {
      None
    } else {
      Some(unsafe { self.get_unchecked(0) })
    }
  }

  /// Returns a mutable reference to the first element of the StaticVec in `Some` if the StaticVec
  /// is not empty, or `None` otherwise.
  #[inline(always)]
  pub fn first_mut(&mut self) -> Option<&mut T> {
    if self.is_empty() {
      None
    } else {
      Some(unsafe { self.get_unchecked_mut(0) })
    }
  }

  /// Returns a constant reference to the last element of the StaticVec in `Some` if the StaticVec
  /// is not empty, or `None` otherwise.
  #[inline(always)]
  pub fn last(&self) -> Option<&T> {
    if self.is_empty() {
      None
    } else {
      Some(unsafe { self.get_unchecked(self.length - 1) })
    }
  }

  /// Returns a mutable reference to the last element of the StaticVec in `Some` if the StaticVec is
  /// not empty, or `None` otherwise.
  #[inline(always)]
  pub fn last_mut(&mut self) -> Option<&mut T> {
    if self.is_empty() {
      None
    } else {
      Some(unsafe { self.get_unchecked_mut(self.length - 1) })
    }
  }

  /// Asserts that `index` is less than the current length of the StaticVec,
  /// and if so removes the value at that position and returns it. Any values
  /// that exist in later positions are shifted to the left.
  #[inline]
  pub fn remove(&mut self, index: usize) -> T {
    // This is mostly the same as how normal Vec implements it.
    let current_length = self.length;
    assert!(index < current_length);
    unsafe {
      let self_ptr = self.mut_ptr_at_unchecked(index);
      let res = self_ptr.read();
      self_ptr
        .offset(1)
        .copy_to(self_ptr, current_length - index - 1);
      self.set_len(current_length - 1);
      res
    }
  }

  /// Removes the first instance of `item` from the StaticVec if the item exists.
  #[inline(always)]
  pub fn remove_item(&mut self, item: &T) -> Option<T>
  where T: PartialEq {
    // Adapted this from normal Vec's implementation.
    if let Some(pos) = self.iter().position(|x| *x == *item) {
      Some(self.remove(pos))
    } else {
      None
    }
  }

  /// Returns `None` if `index` is greater than or equal to the current length of the StaticVec.
  /// Otherwise, removes the value at that position and returns it in `Some`, and then
  /// moves the last value in the StaticVec into the empty slot.
  #[inline(always)]
  pub fn swap_pop(&mut self, index: usize) -> Option<T> {
    if index < self.length {
      unsafe {
        let new_length = self.length - 1;
        let last_value = self.ptr_at_unchecked(new_length).read();
        self.set_len(new_length);
        Some(self.mut_ptr_at_unchecked(index).replace(last_value))
      }
    } else {
      None
    }
  }

  /// Asserts that `index` is less than the current length of the StaticVec,
  /// and if so removes the value at that position and returns it, and then
  /// moves the last value in the StaticVec into the empty slot.
  #[inline(always)]
  pub fn swap_remove(&mut self, index: usize) -> T {
    assert!(index < self.length);
    unsafe {
      let new_length = self.length - 1;
      let last_value = self.ptr_at_unchecked(new_length).read();
      self.set_len(new_length);
      self.mut_ptr_at_unchecked(index).replace(last_value)
    }
  }

  /// Asserts that the current length of the StaticVec is less than `N` and that
  /// `index` is less than the length, and if so inserts `value` at that position.
  /// Any values that exist in positions after `index` are shifted to the right.
  #[inline]
  pub fn insert(&mut self, index: usize, value: T) {
    let old_length = self.length;
    assert!(old_length < N && index <= old_length);
    unsafe {
      let self_ptr = self.mut_ptr_at_unchecked(index);
      self_ptr.copy_to(self_ptr.offset(1), old_length - index);
      self_ptr.write(value);
      self.set_len(old_length + 1);
    }
  }

  /// Functionally equivalent to [`insert`](crate::StaticVec::insert), except with multiple
  /// items provided by an iterator as opposed to just one. This function will return immediately
  /// if / when the StaticVec reaches maximum capacity, regardless of whether the iterator still has
  /// more items to yield.
  ///
  /// For safety reasons, as StaticVec cannot increase in capacity, the
  /// iterator is required to implement [`ExactSizeIterator`](core::iter::ExactSizeIterator)
  /// rather than just [`Iterator`](core::iter::Iterator) (though this function still does
  /// the appropriate checking internally to avoid dangerous outcomes in the event of a blatantly
  /// incorrect [`ExactSizeIterator`](core::iter::ExactSizeIterator) implementation.)
  #[inline]
  pub fn insert_many<I: IntoIterator<Item = T>>(&mut self, index: usize, iter: I)
  where I::IntoIter: ExactSizeIterator<Item = T> {
    let old_length = self.length;
    assert!(
      old_length < N && index <= old_length,
      "Insufficient remaining capacity / out of bounds!"
    );
    let mut it = iter.into_iter();
    if index == old_length {
      return self.extend(it);
    }
    let iter_size = it.len();
    assert!(
      index + iter_size >= index && (old_length - index) + iter_size < N,
      "Insufficient remaining capacity / out of bounds!"
    );
    unsafe {
      let mut self_ptr = self.mut_ptr_at_unchecked(index);
      self_ptr.copy_to(self_ptr.add(iter_size), old_length - index);
      self.length = index;
      let mut item_count = 0;
      while item_count < N {
        if let Some(item) = it.next() {
          let mut current = self_ptr.add(item_count);
          if item_count >= iter_size {
            self_ptr = self.mut_ptr_at_unchecked(index);
            current = self_ptr.add(item_count);
            current.copy_to(current.offset(1), old_length - index);
          }
          current.write(item);
          item_count += 1;
        } else {
          break;
        }
      }
      self.length = old_length + item_count;
    }
  }

  /// Inserts `value` at `index` if the current length of the StaticVec is less than `N` and `index`
  /// is less than the length, or returns a [`CapacityError`](crate::errors::CapacityError)
  /// otherwise. Any values that exist in positions after `index` are shifted to the right.
  #[inline]
  pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), CapacityError<N>> {
    let old_length = self.length;
    if old_length < N && index <= old_length {
      unsafe {
        let self_ptr = self.mut_ptr_at_unchecked(index);
        self_ptr.copy_to(self_ptr.offset(1), old_length - index);
        self_ptr.write(value);
        self.set_len(old_length + 1);
        Ok(())
      }
    } else {
      Err(CapacityError {})
    }
  }

  /// Returns `true` if `value` is present in the StaticVec.
  /// Locally requires that `T` implements [`PartialEq`](core::cmp::PartialEq)
  /// to make it possible to compare the elements of the StaticVec with `value`.
  #[inline(always)]
  pub fn contains(&self, value: &T) -> bool
  where T: PartialEq {
    self.iter().any(|current| current == value)
  }

  /// Removes all contents from the StaticVec and sets its length back to 0.
  #[inline(always)]
  pub fn clear(&mut self) {
    unsafe {
      ptr::drop_in_place(self.as_mut_slice());
    }
    self.length = 0;
  }

  /// Returns a [`StaticVecIterConst`](crate::iterators::StaticVecIterConst) over the StaticVec's
  /// inhabited area.
  #[inline(always)]
  pub fn iter(&self) -> StaticVecIterConst<T, N> {
    StaticVecIterConst {
      start: self.as_ptr(),
      end: match intrinsics::size_of::<T>() {
        0 => (self.as_ptr() as *const u8).wrapping_add(self.length) as *const T,
        _ => unsafe { self.ptr_at_unchecked(self.length) },
      },
      marker: PhantomData,
    }
  }

  /// Returns a [`StaticVecIterMut`](crate::iterators::StaticVecIterMut) over the StaticVec's
  /// inhabited area.
  #[inline(always)]
  pub fn iter_mut(&mut self) -> StaticVecIterMut<T, N> {
    StaticVecIterMut {
      start: self.as_mut_ptr(),
      end: match intrinsics::size_of::<T>() {
        0 => (self.as_mut_ptr() as *mut u8).wrapping_add(self.length) as *mut T,
        _ => unsafe { self.mut_ptr_at_unchecked(self.length) },
      },
      marker: PhantomData,
    }
  }

  /// Returns a separate, stable-sorted StaticVec of the contents of the
  /// StaticVec's inhabited area without modifying the original data.
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to avoid soundness issues,
  /// and [`Ord`](core::cmp::Ord) to make the sorting possible.
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline]
  pub fn sorted(&self) -> Self
  where T: Copy + Ord {
    // StaticVec uses specialization to have an optimized version of `Clone` for copy types.
    let mut res = self.clone();
    res.sort();
    res
  }

  /// Returns a separate, unstable-sorted StaticVec of the contents of the
  /// StaticVec's inhabited area without modifying the original data.
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to avoid soundness issues,
  /// and [`Ord`](core::cmp::Ord) to make the sorting possible.
  #[inline]
  pub fn sorted_unstable(&self) -> Self
  where T: Copy + Ord {
    // StaticVec uses specialization to have an optimized version of `Clone` for copy types.
    let mut res = self.clone();
    res.sort_unstable();
    res
  }

  /// Returns a separate, reversed StaticVec of the contents of the StaticVec's
  /// inhabited area without modifying the original data.
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to avoid soundness issues.
  #[inline(always)]
  pub fn reversed(&self) -> Self
  where T: Copy {
    Self {
      data: reverse_copy(self.length, &self.data),
      length: self.length,
    }
  }

  /// Returns a new StaticVec instance filled with the return value of an initializer function.
  /// The length field of the newly created StaticVec will be equal to its capacity.
  ///
  /// Example usage:
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
    for i in 0..N {
      unsafe {
        res.mut_ptr_at_unchecked(i).write(initializer());
        res.length += 1;
      }
    }
    res
  }

  /// Returns a new StaticVec instance filled with the return value of an initializer function.
  /// Unlike for [`filled_with`](crate::StaticVec::filled_with), the initializer function in
  /// this case must take a single usize variable as an input parameter, which will be called
  /// with the current index of the `0..N` loop that
  /// [`filled_with_by_index`](crate::StaticVec::filled_with_by_index) is implemented with
  /// internally. The length field of the newly created StaticVec will be equal to its capacity.
  ///
  /// Example usage:
  /// ```
  /// let v = StaticVec::<usize, 64>::filled_with_by_index(|i| { i + 1 });
  /// assert_eq!(v.len(), 64);
  /// assert_eq!(v[0], 1);
  /// assert_eq!(v[1], 2);
  /// assert_eq!(v[2], 3);
  /// assert_eq!(v[3], 4);
  /// ```
  #[inline]
  pub fn filled_with_by_index<F>(mut initializer: F) -> Self
  where F: FnMut(usize) -> T {
    let mut res = Self::new();
    for i in 0..N {
      unsafe {
        res.mut_ptr_at_unchecked(i).write(initializer(i));
        res.length += 1;
      }
    }
    res
  }

  /// Copies and appends all elements, if any, of a slice (which can also be `&mut` as it will
  /// coerce implicitly to `&`) to the StaticVec. If the slice has a length greater than the
  /// StaticVec's remaining capacity, any contents after that point are ignored.
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to avoid soundness issues.
  #[inline(always)]
  pub fn extend_from_slice(&mut self, other: &[T])
  where T: Copy {
    let old_length = self.length;
    let added_length = other.len().min(N - old_length);
    // Safety: added_length is <= our remaining capacity and other.len.
    unsafe {
      other
        .as_ptr()
        .copy_to_nonoverlapping(self.mut_ptr_at_unchecked(old_length), added_length);
      self.set_len(old_length + added_length);
    }
  }

  /// Copies and appends all elements, if any, of a slice to the StaticVec if the
  /// StaticVec's remaining capacity is greater than the length of the slice, or returns
  /// a [`CapacityError`](crate::errors::CapacityError) otherwise.
  #[inline(always)]
  pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), CapacityError<N>>
  where T: Copy {
    let old_length = self.length;
    let added_length = other.len();
    if N - old_length < added_length {
      return Err(CapacityError {});
    }
    unsafe {
      other
        .as_ptr()
        .copy_to_nonoverlapping(self.mut_ptr_at_unchecked(old_length), added_length);
      self.set_len(old_length + added_length);
    }
    Ok(())
  }

  /// Appends `self.remaining_capacity()` (or as many as available) items from
  /// `other` to `self`. The appended items (if any) will no longer exist in `other` afterwards,
  /// as `other`'s `length` field will be adjusted to indicate.
  ///
  /// The `N2` parameter does not need to be provided explicitly, and can be inferred directly from
  /// the constant `N2` constraint of `other` (which may or may not be the same as the `N`
  /// constraint of `self`.)
  #[inline]
  pub fn append<const N2: usize>(&mut self, other: &mut StaticVec<T, N2>) {
    let old_length = self.length;
    let item_count = (N - old_length).min(other.length);
    let other_new_length = other.length - item_count;
    unsafe {
      self
        .mut_ptr_at_unchecked(old_length)
        .copy_from_nonoverlapping(other.as_ptr(), item_count);
      other
        .as_mut_ptr()
        .copy_from(other.ptr_at_unchecked(item_count), other_new_length);
      other.set_len(other_new_length);
      self.set_len(old_length + item_count);
    }
  }

  /// Returns a new StaticVec consisting of the elements of `self` and `other` concatenated in
  /// linear fashion such that the first element of `other` comes immediately after the last
  /// element of `self`.
  ///
  /// The `N2` parameter does not need to be provided explicitly, and can be inferred directly from
  /// the constant `N2` constraint of `other` (which may or may not be the same as the `N`
  /// constraint of `self`.)
  ///
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to
  /// avoid soundness issues and also allow for a more efficient implementation than would otherwise
  /// be possible.
  ///
  /// Example usage:
  /// ```
  /// assert_eq!(
  ///  staticvec!["A, B"].concat(&staticvec!["C", "D", "E", "F"]),
  ///  ["A, B", "C", "D", "E", "F"]
  /// );
  /// ```
  #[inline]
  pub fn concat<const N2: usize>(&self, other: &StaticVec<T, N2>) -> StaticVec<T, { N + N2 }>
  where T: Copy {
    let length = self.length;
    let other_length = other.length;
    let mut res = StaticVec::new();
    unsafe {
      self
        .as_ptr()
        .copy_to_nonoverlapping(res.as_mut_ptr(), length);
      other
        .as_ptr()
        .copy_to_nonoverlapping(res.mut_ptr_at_unchecked(length), other_length);
      res.set_len(length + other_length);
    }
    res
  }

  /// A version of [`concat`](crate::StaticVec::concat) for scenarios where `T` does not
  /// derive [`Copy`](core::marker::Copy) but does implement [`Clone`](core::clone::Clone).
  ///
  /// Due to needing to call `clone()` through each individual element of `self` and `other`, this
  /// function is less efficient than [`concat`](crate::StaticVec::concat), so
  /// [`concat`](crate::StaticVec::concat) should be preferred whenever possible.
  #[inline]
  pub fn concat_clone<const N2: usize>(
    &self,
    other: &StaticVec<T, N2>,
  ) -> StaticVec<T, { N + N2 }>
  where
    T: Clone,
  {
    let mut res = StaticVec::new();
    for i in 0..self.length {
      unsafe { res.push_unchecked(self.get_unchecked(i).clone()) };
    }
    for i in 0..other.length {
      unsafe { res.push_unchecked(other.get_unchecked(i).clone()) };
    }
    res
  }

  /// Returns a new StaticVec consisting of the elements of `self` in linear order, interspersed
  /// with a copy of `separator` between each one.
  ///
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to
  /// avoid soundness issues and also allow for a more efficient implementation than would otherwise
  /// be possible.
  ///
  /// Example usage:
  /// ```
  /// assert_eq!(
  ///  staticvec!["A", "B", "C", "D"].intersperse("Z"),
  ///  ["A, "Z", B", "Z", "C", "Z", "D"]
  /// );
  /// ```
  #[inline]
  pub fn intersperse(&self, separator: T) -> StaticVec<T, { N * 2 }>
  where T: Copy {
    if self.is_empty() {
      return StaticVec::new();
    }
    let mut res = StaticVec::new();
    // The `as *mut T` cast here is necessary to make the type
    // inference work properly (at the moment at least.) `rustc` still gets
    // a bit confused by math operations done on const generic values
    // in return types it seems.
    let mut res_ptr = res.as_mut_ptr() as *mut T;
    let mut i = 0;
    let length = self.length;
    while i < length - 1 {
      unsafe {
        res_ptr.write(self.ptr_at_unchecked(i).read());
        res_ptr.offset(1).write(separator);
        res_ptr = res_ptr.offset(2);
        i += 1
      }
    }
    unsafe {
      res_ptr.write(self.ptr_at_unchecked(i).read());
      res.set_len((length * 2) - 1);
    }
    res
  }

  /// A version of [`intersperse`](crate::StaticVec::intersperse) for scenarios where `T` does not
  /// derive [`Copy`](core::marker::Copy) but does implement [`Clone`](core::clone::Clone).
  ///
  /// Due to needing to call `clone()` through each individual element of `self` and also on
  /// `separator`, this function is less efficient than
  /// [`intersperse`](crate::StaticVec::intersperse), so
  /// [`intersperse`](crate::StaticVec::intersperse) should be preferred whenever possible.
  #[inline]
  pub fn intersperse_clone(&self, separator: T) -> StaticVec<T, { N * 2 }>
  where T: Clone {
    if self.is_empty() {
      return StaticVec::new();
    }
    let mut res = StaticVec::new();
    let length = self.length;
    unsafe {
      for item in self.as_slice().get_unchecked(0..length - 1) {
        res.push_unchecked(item.clone());
        res.push_unchecked(separator.clone());
      }
      res.push_unchecked(self.get_unchecked(length - 1).clone());
    }
    res
  }

  /// Returns a StaticVec containing the contents of a [`Vec`](alloc::vec::Vec) instance.
  /// If the [`Vec`](alloc::vec::Vec) has a length greater than the declared capacity of the
  /// resulting StaticVec, any contents after that point are ignored. Note that using this function
  /// consumes the source [`Vec`](alloc::vec::Vec).
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline]
  pub fn from_vec(mut vec: Vec<T>) -> Self {
    let vec_len = vec.len();
    let item_count = vec_len.min(N);
    Self {
      data: {
        unsafe { vec.set_len(0) };
        let mut data = Self::new_data_uninit();
        unsafe {
          vec
            .as_ptr()
            .copy_to_nonoverlapping(Self::first_ptr_mut(&mut data), item_count);
          // Manually drop any excess values in the source vec to avoid undesirable memory leaks.
          if vec_len > item_count {
            ptr::drop_in_place(slice_from_raw_parts_mut(
              vec.as_mut_ptr().add(item_count),
              vec_len - item_count,
            ));
          }
          data
        }
      },
      length: item_count,
    }
  }

  /// Returns a [`Vec`](alloc::vec::Vec) containing the contents of the StaticVec instance.
  /// The returned [`Vec`](alloc::vec::Vec) will initially have the same value for
  /// [`len`](alloc::vec::Vec::len) and [`capacity`](alloc::vec::Vec::capacity) as the source
  /// StaticVec. Note that using this function consumes the source StaticVec.
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  pub fn into_vec(mut self) -> Vec<T> {
    let mut res = Vec::with_capacity(N);
    let length = self.length;
    unsafe {
      self
        .as_ptr()
        .copy_to_nonoverlapping(res.as_mut_ptr(), length);
      res.set_len(length);
      self.set_len(0);
      res
    }
  }

  /// Removes the specified range of elements from the StaticVec and returns them in a new one.
  #[inline]
  pub fn drain<R>(&mut self, range: R) -> Self
  // No Copy bounds here because the original StaticVec gives up all access to the values in
  // question.
  where R: RangeBounds<usize> {
    // Borrowed this part from normal Vec's implementation.
    let current_length = self.length;
    let start = match range.start_bound() {
      Included(&idx) => idx,
      Excluded(&idx) => idx + 1,
      Unbounded => 0,
    };
    let end = match range.end_bound() {
      Included(&idx) => idx + 1,
      Excluded(&idx) => idx,
      Unbounded => current_length,
    };
    assert!(start <= end && end <= current_length);
    let res_length = end - start;
    Self {
      data: {
        let mut res = Self::new_data_uninit();
        unsafe {
          self
            .ptr_at_unchecked(start)
            .copy_to_nonoverlapping(Self::first_ptr_mut(&mut res), res_length);
          self
            .ptr_at_unchecked(end)
            .copy_to(self.mut_ptr_at_unchecked(start), current_length - end);
          self.set_len(current_length - res_length);
          res
        }
      },
      length: res_length,
    }
  }

  /// Removes the specified range of elements from the StaticVec and returns them in a
  /// [`StaticVecDrain`](crate::iterators::StaticVecDrain).
  #[inline]
  pub fn drain_iter<R>(&mut self, range: R) -> StaticVecDrain<T, N>
  where R: RangeBounds<usize> {
    // Borrowed this part from normal Vec's implementation.
    let length = self.length;
    let start = match range.start_bound() {
      Included(&idx) => idx,
      Excluded(&idx) => idx + 1,
      Unbounded => 0,
    };
    let end = match range.end_bound() {
      Included(&idx) => idx + 1,
      Excluded(&idx) => idx,
      Unbounded => length,
    };
    assert!(start <= end && end <= length);
    unsafe {
      // Set the length to 0 to avoid memory issues if anything goes wrong with
      // the Drain.
      self.set_len(start);
      // Create the StaticVecDrain from the specified range.
      StaticVecDrain {
        start: end,
        length: length - end,
        iter: StaticVecIterConst {
          start: self.ptr_at_unchecked(start),
          end: match intrinsics::size_of::<T>() {
            0 => (self.as_ptr() as *const u8).wrapping_add(end) as *const T,
            _ => self.ptr_at_unchecked(end),
          },
          marker: PhantomData,
        },
        vec: self,
      }
    }
  }

  /// Removes all elements in the StaticVec for which `filter` returns true and
  /// returns them in a new one.
  #[inline]
  pub fn drain_filter<F>(&mut self, mut filter: F) -> Self
  where F: FnMut(&mut T) -> bool {
    let old_length = self.length;
    // Temporarily set our length to 0 to avoid double drops and such if anything
    // goes wrong in the filter loop.
    self.length = 0;
    let mut res = Self::new();
    let mut res_length = 0;
    unsafe {
      // If `self.length` was already 0, this loop is skipped completely.
      for i in 0..old_length {
        // This is fine because we intentionally set `self.length` to `0` ourselves just now.
        if filter(self.get_unchecked_mut(i)) {
          res
            .mut_ptr_at_unchecked(res_length)
            .write(self.ptr_at_unchecked(i).read());
          res_length += 1;
        } else if res_length > 0 {
          self
            .ptr_at_unchecked(i)
            .copy_to_nonoverlapping(self.mut_ptr_at_unchecked(i - res_length), 1);
        }
      }
    }
    self.length = old_length - res_length;
    res.length = res_length;
    res
  }

  /// Removes all elements in the StaticVec for which `filter` returns false.
  #[inline(always)]
  pub fn retain<F>(&mut self, mut filter: F)
  where F: FnMut(&T) -> bool {
    self.drain_filter(|val| !filter(val));
  }

  /// Shortens the StaticVec, keeping the first `length` elements and dropping the rest.
  /// Does nothing if `length` is greater than or equal to the current length of the StaticVec.
  #[inline(always)]
  pub fn truncate(&mut self, length: usize) {
    if length < self.length {
      let old_length = self.length;
      unsafe {
        self.set_len(length);
        ptr::drop_in_place(
          slice_from_raw_parts_mut(self.mut_ptr_at_unchecked(length), old_length - length)
        );
      }
    }
  }

  /// Splits the StaticVec into two at the given index.
  /// The original StaticVec will contain elements `0..at`,
  /// and the new one will contain elements `at..length`.
  #[inline]
  pub fn split_off(&mut self, at: usize) -> Self {
    let length = self.length;
    assert!(at <= length);
    let split_length = length - at;
    Self {
      data: unsafe {
        self.set_len(at);
        let mut split = Self::new_data_uninit();
        self
          .ptr_at_unchecked(at)
          .copy_to_nonoverlapping(Self::first_ptr_mut(&mut split), split_length);
        split
      },
      length: split_length,
    }
  }

  /// Removes all but the first of consecutive elements in the StaticVec satisfying a given equality
  /// relation.
  #[inline(always)]
  pub fn dedup_by<F>(&mut self, same_bucket: F)
  where F: FnMut(&mut T, &mut T) -> bool {
    // Mostly the same as Vec's version.
    let new_length = self.as_mut_slice().partition_dedup_by(same_bucket).0.len();
    self.truncate(new_length);
  }

  /// Removes consecutive repeated elements in the StaticVec according to the
  /// locally required [`PartialEq`](core::cmp::PartialEq) trait implementation for `T`.
  #[inline(always)]
  pub fn dedup(&mut self)
  where T: PartialEq {
    // Exactly the same as Vec's version.
    self.dedup_by(|a, b| a == b)
  }

  /// Removes all but the first of consecutive elements in the StaticVec that
  /// resolve to the same key.
  #[inline(always)]
  pub fn dedup_by_key<F, K>(&mut self, mut key: F)
  where
    F: FnMut(&mut T) -> K,
    K: PartialEq<K>, {
    // Exactly the same as Vec's version.
    self.dedup_by(|a, b| key(a) == key(b))
  }

  /// Returns a new StaticVec representing the difference of `self` and `other` (that is,
  /// all items present in `self`, but *not* present in `other`.)
  ///
  /// The `N2` parameter does not need to be provided explicitly, and can be inferred from `other`
  /// itself.
  ///
  /// Locally requires that `T` implements [`Clone`](core::clone::Clone) to avoid soundness issues
  /// while accommodating for more types than [`Copy`](core::marker::Copy) would appropriately for
  /// this function, and [`PartialEq`](core::cmp::PartialEq) to make the item comparisons possible.
  ///
  /// Example usage:
  /// ```
  /// assert_eq!(
  ///   staticvec![4, 5, 6, 7].difference(&staticvec![1, 2, 3, 7]),
  ///   [4, 5, 6]
  /// );
  /// ```
  #[inline]
  pub fn difference<const N2: usize>(&self, other: &StaticVec<T, N2>) -> Self
  where T: Clone + PartialEq {
    let mut res = Self::new();
    for left in self {
      let mut found = false;
      for right in other {
        if left == right {
          found = true;
          break;
        }
      }
      if !found {
        unsafe { res.push_unchecked(left.clone()) }
      }
    }
    res
  }

  /// Returns a new StaticVec representing the symmetric difference of `self` and `other` (that is,
  /// all items present in at least one of `self` or `other`, but *not* present in both.)
  ///
  /// The `N2` parameter does not need to be provided explicitly, and can be inferred from `other`
  /// itself.
  ///
  /// Locally requires that `T` implements [`Clone`](core::clone::Clone) to avoid soundness issues
  /// while accommodating for more types than [`Copy`](core::marker::Copy) would appropriately for
  /// this function, and [`PartialEq`](core::cmp::PartialEq) to make the item comparisons possible.
  ///
  /// Example usage:
  /// ```
  /// assert_eq!(
  ///   staticvec![1, 2, 3].symmetric_difference(&staticvec![3, 4, 5]),
  ///   [1, 2, 4, 5]
  /// );
  /// ```
  #[inline]
  pub fn symmetric_difference<const N2: usize>(
    &self,
    other: &StaticVec<T, N2>,
  ) -> StaticVec<T, { N + N2 }>
  where
    T: Clone + PartialEq,
  {
    let mut res = StaticVec::new();
    for left in self {
      let mut found = false;
      for right in other {
        if left == right {
          found = true;
          break;
        }
      }
      if !found {
        unsafe { res.push_unchecked(left.clone()) }
      }
    }
    for right in other {
      let mut found = false;
      for left in self {
        if right == left {
          found = true;
          break;
        }
      }
      if !found {
        unsafe { res.push_unchecked(right.clone()) }
      }
    }
    res
  }

  /// Returns a new StaticVec representing the intersection of `self` and `other` (that is,
  /// all items present in both `self` and `other`.)
  ///
  /// The `N2` parameter does not need to be provided explicitly, and can be inferred from `other`
  /// itself.
  ///
  /// Locally requires that `T` implements [`Clone`](core::clone::Clone) to avoid soundness issues
  /// while accommodating for more types than [`Copy`](core::marker::Copy) would appropriately for
  /// this function, and [`PartialEq`](core::cmp::PartialEq) to make the item comparisons possible.
  ///
  /// Example usage:
  /// ```
  /// assert_eq!(
  ///   staticvec![4, 5, 6, 7].intersection(&staticvec![1, 2, 3, 7, 4]),
  ///   [4, 7],
  /// );
  /// ```
  #[inline]
  pub fn intersection<const N2: usize>(&self, other: &StaticVec<T, N2>) -> Self
  where T: Clone + PartialEq {
    let mut res = Self::new();
    for left in self {
      let mut found = false;
      for right in other {
        if left == right {
          found = true;
          break;
        }
      }
      if found && !res.contains(left) {
        unsafe { res.push_unchecked(left.clone()) }
      }
    }
    res
  }

  /// Returns a new StaticVec representing the union of `self` and `other` (that is, the full
  /// contents of both `self` and `other`, minus any duplicates.)
  ///
  /// The `N2` parameter does not need to be provided explicitly, and can be inferred from `other`
  /// itself.
  ///
  /// Locally requires that `T` implements [`Clone`](core::clone::Clone) to avoid soundness issues
  /// while accommodating for more types than [`Copy`](core::marker::Copy) would appropriately for
  /// this function, and [`PartialEq`](core::cmp::PartialEq) to make the item comparisons possible.
  ///
  /// Example usage:
  /// ```
  /// assert_eq!(
  ///   staticvec![1, 2, 3].union(&staticvec![4, 2, 3, 4]),
  ///   [1, 2, 3, 4],
  /// );
  /// ```
  #[inline]
  pub fn union<const N2: usize>(&self, other: &StaticVec<T, N2>) -> StaticVec<T, { N + N2 }>
  where T: Clone + PartialEq {
    if self.length <= other.length {
      let mut res = StaticVec::from_iter(self.iter().chain(other.difference(self).iter()).cloned());
      res.dedup();
      res
    } else {
      let mut res = StaticVec::from_iter(other.iter().chain(self.difference(other).iter()).cloned());
      res.dedup();
      res
    }
  }

  /// A concept borrowed from the widely-used `SmallVec` crate, this function
  /// returns a tuple consisting of a constant pointer to the first element of the StaticVec,
  /// the length of the StaticVec, and the capacity of the StaticVec.
  #[inline(always)]
  pub const fn triple(&self) -> (*const T, usize, usize) {
    (self.as_ptr(), self.length, N)
  }

  /// A mutable version of [`triple`](crate::StaticVec::triple). This implementation differs from
  /// the one found in `SmallVec` in that it only provides the first element of the StaticVec as
  /// a mutable pointer, not also the length as a mutable reference.
  #[inline(always)]
  pub fn triple_mut(&mut self) -> (*mut T, usize, usize) {
    (self.as_mut_ptr(), self.length, N)
  }

  /// Linearly adds (in a mathematical sense) the contents of two same-capacity
  /// StaticVecs and returns the results in a new one of equal capacity.
  ///
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to allow
  /// for an efficient implementation, and [`Add`](core::ops::Add) to make it possible
  /// to add the elements.
  ///
  /// For both performance and safety reasons, this function requires that both `self`
  /// and `other` are at full capacity, and will panic if that is not the case (that is,
  /// if `self.is_full() && other.is_full()` is not equal to `true`.)
  ///
  /// Example usage:
  /// ```
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.added(&B), [6.0, 8.0, 10.0, 12.0]);
  /// ```
  #[inline(always)]
  pub fn added(&self, other: &Self) -> Self
  where T: Copy + Add<Output = T> {
    assert!(self.is_full() && other.is_full());
    let mut res = Self::new();
    for i in 0..N {
      unsafe {
        res
          .mut_ptr_at_unchecked(i)
          .write(*self.get_unchecked(i) + *other.get_unchecked(i));
      }
    }
    res.length = N;
    res
  }

  /// Linearly subtracts (in a mathematical sense) the contents of two same-capacity
  /// StaticVecs and returns the results in a new one of equal capacity.
  ///
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to allow
  /// for an efficient implementation, and [`Sub`](core::ops::Sub) to make it possible
  /// to subtract the elements.
  ///
  /// For both performance and safety reasons, this function requires that both `self`
  /// and `other` are at full capacity, and will panic if that is not the case (that is,
  /// if `self.is_full() && other.is_full()` is not equal to `true`.)
  ///
  /// Example usage:
  /// ```
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.subtracted(&B), [2.0, 2.0, 2.0, 2.0]);
  /// ```
  #[inline(always)]
  pub fn subtracted(&self, other: &Self) -> Self
  where T: Copy + Sub<Output = T> {
    assert!(self.is_full() && other.is_full());
    let mut res = Self::new();
    for i in 0..N {
      unsafe {
        res
          .mut_ptr_at_unchecked(i)
          .write(*self.get_unchecked(i) - *other.get_unchecked(i));
      }
    }
    res.length = N;
    res
  }

  /// Linearly multiplies (in a mathematical sense) the contents of two same-capacity
  /// StaticVecs and returns the results in a new one of equal capacity.
  ///
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to allow
  /// for an efficient implementation, and [`Mul`](core::ops::Mul) to make it possible
  /// to multiply the elements.
  ///
  /// For both performance and safety reasons, this function requires that both `self`
  /// and `other` are at full capacity, and will panic if that is not the case (that is,
  /// if `self.is_full() && other.is_full()` is not equal to `true`.)
  ///
  /// Example usage:
  /// ```
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.multiplied(&B), [8.0, 15.0, 24.0, 35.0]);
  /// ```
  #[inline(always)]
  pub fn multiplied(&self, other: &Self) -> Self
  where T: Copy + Mul<Output = T> {
    assert!(self.is_full() && other.is_full());
    let mut res = Self::new();
    for i in 0..N {
      unsafe {
        res
          .mut_ptr_at_unchecked(i)
          .write(*self.get_unchecked(i) * *other.get_unchecked(i));
      }
    }
    res.length = N;
    res
  }

  /// Linearly divides (in a mathematical sense) the contents of two same-capacity
  /// StaticVecs and returns the results in a new one of equal capacity.
  ///
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to allow
  /// for an efficient implementation, and [`Div`](core::ops::Div) to make it possible
  /// to divide the elements.
  ///
  /// For both performance and safety reasons, this function requires that both `self`
  /// and `other` are at full capacity, and will panic if that is not the case (that is,
  /// if `self.is_full() && other.is_full()` is not equal to `true`.)
  ///
  /// Example usage:
  /// ```
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.divided(&B), [2.0, 1.6666666666666667, 1.5, 1.4]);
  /// ```
  #[inline(always)]
  pub fn divided(&self, other: &Self) -> Self
  where T: Copy + Div<Output = T> {
    assert!(self.is_full() && other.is_full());
    let mut res = Self::new();
    for i in 0..N {
      unsafe {
        res
          .mut_ptr_at_unchecked(i)
          .write(*self.get_unchecked(i) / *other.get_unchecked(i));
      }
    }
    res.length = N;
    res
  }

  /// An internal convenience function to get an *uninitialized* instance of
  /// `MaybeUninit<[T; N]>`.
  #[inline(always)]
  pub(crate) const fn new_data_uninit() -> MaybeUninit<[T; N]> {
    MaybeUninit::uninit()
  }

  /// An internal convenience function to go from `&MaybeUninit<[T; N]>` to `*const T`.
  /// Similar to [`MaybeUninit::first_ptr`](core::mem::MaybeUninit::first_ptr), but for arrays
  /// as opposed to slices.
  #[inline(always)]
  pub(crate) const fn first_ptr(this: &MaybeUninit<[T; N]>) -> *const T {
    this as *const MaybeUninit<[T; N]> as *const T
  }

  /// An internal convenience function to go from `&mut MaybeUninit<[T; N]>` to `*mut T`.
  /// Similar to [`MaybeUninit::first_ptr_mut`](core::mem::MaybeUninit::first_ptr_mut), but for
  /// arrays as opposed to slices.
  #[inline(always)]
  pub(crate) const fn first_ptr_mut(this: &mut MaybeUninit<[T; N]>) -> *mut T {
    this as *mut MaybeUninit<[T; N]> as *mut T
  }
}
