//! **Note:** the complete list of things **not** available when using `default-features = false`
//! for `#![no_std]` compatibility is as follows:
//! - [`StaticVec::sorted`]
//! - [`StaticVec::into_vec`] (and the corresponding [`Into`] impl)
//! - [`StaticVec::from_vec`] (and the corresponding [`From`] impl)
//! - the implementation of the [`Read`](std::io::Read) trait for [`StaticVec`]
//! - the implementation of the [`BufRead`](std::io::BufRead) trait for [`StaticVec`]
//! - the implementation of the [`io::Write`](std::io::Write) trait for [`StaticVec`]
//! - the implementation of [`From`] for [`StaticString`](crate::string::StaticString) from
//!   [`String`](alloc::string::String)
//! - the implementations of [`PartialEq`] and [`PartialOrd`] against
//!   [`String`](alloc::string::String) for [`StaticString`](crate::string::StaticString)
//! - the implementation of [`Error`](std::error::Error) for [`StringError`]
//! - the `bounds_to_string` function unique to this crate and implemented by several of the
//!   iterators in it

#![no_std]
#![allow(
  // Clippy wants every single instance of the word "StaticVec" to be in syntax-highlight
  // backticks, which IMO looks way too "noisy" when actually rendered.
  clippy::doc_markdown,
  // Clippy thinks inline always is a bad idea even for the most simple of one-liners, so
  // IMO it's just not a particularly helpful lint.
  clippy::inline_always,
  // The "if-let" syntax Clippy recommends as an alternative to "match" in this lint is
  // generally way less readable IMO.
  clippy::match_bool,
  // Without this, every single use of const generics is warned against.
  incomplete_features
)]
#![feature(
  const_fn,
  const_fn_union,
  const_generics,
  const_mut_refs,
  const_panic,
  const_ptr_offset_from,
  const_raw_ptr_deref,
  const_raw_ptr_to_usize_cast,
  const_slice_from_raw_parts,
  const_trait_impl,
  core_intrinsics,
  doc_cfg,
  exact_size_is_empty,
  maybe_uninit_extra,
  maybe_uninit_ref,
  maybe_uninit_uninit_array,
  slice_partition_dedup,
  specialization,
  trusted_len,
  untagged_unions
)]
#![cfg_attr(feature = "std", feature(read_initializer))]

use core::cmp::{Ord, PartialEq};
use core::intrinsics;
#[doc(hidden)]
pub use core::iter::FromIterator;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{
  Add, Bound::Excluded, Bound::Included, Bound::Unbounded, Div, Mul, RangeBounds, Sub,
};
use core::ptr;

pub use crate::errors::{CapacityError, PushCapacityError};
pub use crate::heap::{
  StaticHeap, StaticHeapDrainSorted, StaticHeapIntoIterSorted, StaticHeapPeekMut,
};
pub use crate::iterators::{
  StaticVecDrain, StaticVecIntoIter, StaticVecIterConst, StaticVecIterMut, StaticVecSplice,
};
pub use crate::string::{string_utils, StaticString, StringError};
use crate::utils::{
  quicksort_internal, reverse_copy, slice_from_raw_parts, slice_from_raw_parts_mut,
};

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
#[doc(hidden)]
mod heap;
#[doc(hidden)]
mod string;
pub(crate) mod trait_impls;
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticVec;
  /// let v = StaticVec::<i32, 4>::new();
  /// assert_eq!(v.len(), 0);
  /// assert_eq!(v.capacity(), 4);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let v = StaticVec::<i32, 8>::new_from_slice(&[1, 2, 3]);
  /// assert_eq!(v, [1, 2, 3]);
  /// ```
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
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticVec;
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
  /// # use staticvec::StaticVec;
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
            // then manually drop any excess ones. From the assembly output I've looked
            // at, the compiler interprets this whole sequence in a way that doesn't result
            // in any excess copying, so there should be no performance concerns for larger
            // input arrays.
            let mut forgotten = MaybeUninit::new(values);
            ptr::drop_in_place(forgotten.assume_init_mut().get_unchecked_mut(N2.min(N)..N2));
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const v: StaticVec<i32, 4> = StaticVec::new_from_const_array([1, 2, 3, 4]);
  /// assert_eq!(v, staticvec![1, 2, 3, 4]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(staticvec![1].len(), 1);
  /// ```
  #[inline(always)]
  pub const fn len(&self) -> usize {
    self.length
  }

  /// Returns the total capacity of the StaticVec.
  /// This is always equivalent to the generic `N` parameter it was declared with, which determines
  /// the fixed size of the backing array.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(StaticVec::<usize, 800>::new().capacity(), 800);
  /// ```
  #[inline(always)]
  pub const fn capacity(&self) -> usize {
    N
  }

  /// Does the same thing as [`capacity`](crate::StaticVec::capacity), but as an associated function
  /// rather than a method.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(StaticVec::<f64, 12>::cap(), 12)
  /// ```
  #[inline(always)]
  pub const fn cap() -> usize {
    N
  }

  /// Serves the same purpose as [`capacity`](crate::StaticVec::capacity), but as an associated
  /// constant rather than a method.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(StaticVec::<f64, 12>::CAPACITY, 12)
  /// ```
  pub const CAPACITY: usize = N;

  /// Returns the remaining capacity (which is to say, `self.capacity() - self.len()`) of the
  /// StaticVec.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut vec = StaticVec::<i32, 100>::new();
  /// vec.push(1);
  /// assert_eq!(vec.remaining_capacity(), 99);
  /// ```
  #[inline(always)]
  pub const fn remaining_capacity(&self) -> usize {
    N - self.length
  }

  /// Returns the total size of the inhabited part of the StaticVec (which may be zero if it has a
  /// length of zero or contains ZSTs) in bytes. Specifically, the return value of this function
  /// amounts to a calculation of `size_of::<T>() * self.len()`.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let x = StaticVec::<u8, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  /// assert_eq!(x.size_in_bytes(), 8);
  /// let y = StaticVec::<u16, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  /// assert_eq!(y.size_in_bytes(), 16);
  /// let z = StaticVec::<u32, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  /// assert_eq!(z.size_in_bytes(), 32);
  /// let w = StaticVec::<u64, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  /// assert_eq!(w.size_in_bytes(), 64);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut vec = StaticVec::<i32, 12>::new();
  /// let data = staticvec![1, 2, 3, 4];
  /// unsafe {
  ///   data.as_ptr().copy_to_nonoverlapping(vec.as_mut_ptr(), 4);
  ///   vec.set_len(4);
  /// }
  /// assert_eq!(vec.len(), 4);
  /// assert_eq!(vec.remaining_capacity(), 8);
  /// assert_eq!(vec, data);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert!(StaticVec::<i32, 4>::new().is_empty());
  /// ```
  #[inline(always)]
  pub const fn is_empty(&self) -> bool {
    self.length == 0
  }

  /// Returns true if the current length of the StaticVec is greater than 0.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert!(staticvec![staticvec![1, 1], staticvec![2, 2]].is_not_empty());
  /// ```
  // Clippy wants `!is_empty()` for this, but I prefer it as-is. My question is though, does it
  // actually know that we have an applicable `is_empty()` function, or is it just guessing? I'm not
  // sure.
  #[allow(clippy::len_zero)]
  #[inline(always)]
  pub const fn is_not_empty(&self) -> bool {
    self.length > 0
  }

  /// Returns true if the current length of the StaticVec is equal to its capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert!(StaticVec::<i32, 4>::filled_with(|| 2).is_full());
  /// ```
  #[inline(always)]
  pub const fn is_full(&self) -> bool {
    self.length == N
  }

  /// Returns true if the current length of the StaticVec is less than its capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert!(StaticVec::<i32, 4>::new().is_not_full());
  /// ```
  #[inline(always)]
  pub const fn is_not_full(&self) -> bool {
    self.length < N
  }

  /// Returns a constant pointer to the first element of the StaticVec's internal array.
  /// It is up to the caller to ensure that the StaticVec lives for as long as they intend
  /// to make use of the returned pointer, as once the StaticVec is dropped the pointer will
  /// point to uninitialized or "garbage" memory.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let v = staticvec!['A', 'B', 'C'];
  /// let p = v.as_ptr();
  /// unsafe { assert_eq!(*p, 'A') };
  /// ```
  #[inline(always)]
  pub const fn as_ptr(&self) -> *const T {
    Self::first_ptr(&self.data)
  }

  /// Returns a mutable pointer to the first element of the StaticVec's internal array.
  /// It is up to the caller to ensure that the StaticVec lives for as long as they intend
  /// to make use of the returned pointer, as once the StaticVec is dropped the pointer will
  /// point to uninitialized or "garbage" memory.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec!['A', 'B', 'C'];
  /// let p = v.as_mut_ptr();
  /// unsafe { *p = 'X' };
  /// assert_eq!(v, ['X', 'B', 'C']);
  /// ```
  #[inline(always)]
  pub const fn as_mut_ptr(&mut self) -> *mut T {
    Self::first_ptr_mut(&mut self.data)
  }

  /// Returns a constant reference to a slice of the StaticVec's inhabited area.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(staticvec![1, 2, 3].as_slice(), &[1, 2, 3]);
  /// ```
  #[inline(always)]
  pub const fn as_slice(&self) -> &[T] {
    // Safety: `self.as_ptr()` is a pointer to an array for which the first `length`
    // elements are guaranteed to be initialized. Therefore this is a valid slice.
    slice_from_raw_parts(self.as_ptr(), self.length)
  }

  /// Returns a mutable reference to a slice of the StaticVec's inhabited area.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![4, 5, 6];
  /// let s = v.as_mut_slice();
  /// s[1] = 9;
  /// assert_eq!(v, [4, 9, 6]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let v = staticvec!["I", "am", "a", "StaticVec!"];
  /// unsafe {
  ///   let p = v.ptr_at_unchecked(3);
  ///   assert_eq!(*p, "StaticVec!");
  /// }
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec!["I", "am", "not a", "StaticVec!"];
  /// unsafe {
  ///   let p = v.mut_ptr_at_unchecked(2);
  ///   *p = "a";
  /// }
  /// assert_eq!(v, ["I", "am", "a", "StaticVec!"]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let v = staticvec!["I", "am", "a", "StaticVec!"];
  /// let p = v.ptr_at(3);
  /// unsafe { assert_eq!(*p, "StaticVec!") };
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec!["I", "am", "not a", "StaticVec!"];
  /// let p = v.mut_ptr_at(2);
  /// unsafe { *p = "a" };
  /// assert_eq!(v, ["I", "am", "a", "StaticVec!"]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// unsafe { assert_eq!(*staticvec![1, 2, 3].get_unchecked(1), 2) };
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 3];
  /// let p = unsafe { v.get_unchecked_mut(1) };
  /// *p = 9;
  /// assert_eq!(v, [1, 9, 3]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<i32, 4>::from([1, 2]);
  /// unsafe { v.push_unchecked(3) };
  /// assert_eq!(v, [1, 2, 3]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<i32, 4>::from([1, 2, 3, 4]);
  /// unsafe { v.pop_unchecked() };
  /// assert_eq!(v, [1, 2, 3]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v1 = StaticVec::<usize, 128>::filled_with_by_index(|i| i * 4);
  /// assert!(v1.try_push(999).is_err());
  /// let mut v2 = StaticVec::<usize, 128>::new();
  /// assert!(v2.try_push(1).is_ok());
  /// ```
  #[inline(always)]
  pub fn try_push(&mut self, value: T) -> Result<(), PushCapacityError<T, N>> {
    if self.is_not_full() {
      unsafe { self.push_unchecked(value) };
      Ok(())
    } else {
      Err(PushCapacityError::new(value))
    }
  }

  /// Pushes a value to the end of the StaticVec. Panics if the collection is
  /// full; that is, if `self.len() == self.capacity()`.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<i32, 8>::new();
  /// v.push(1);
  /// v.push(2);
  /// assert_eq!(v, [1, 2]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 3, 4];
  /// assert_eq!(v.pop(), Some(4));
  /// assert_eq!(v.pop(), Some(3));
  /// assert_eq!(v, [1, 2]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let v1 = staticvec![10, 40, 30];
  /// assert_eq!(Some(&10), v1.first());
  /// let v2 = StaticVec::<i32, 0>::new();
  /// assert_eq!(None, v2.first());
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut x = staticvec![0, 1, 2];
  /// if let Some(first) = x.first_mut() {
  ///   *first = 5;
  /// }
  /// assert_eq!(x, &[5, 1, 2]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let v = staticvec![10, 40, 30];
  /// assert_eq!(Some(&30), v.last());
  /// let w = StaticVec::<i32, 0>::new();
  /// assert_eq!(None, w.last());
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut x = staticvec![0, 1, 2];
  /// if let Some(last) = x.last_mut() {
  ///   *last = 10;
  /// }
  /// assert_eq!(x, &[0, 1, 10]);
  /// ```
  #[inline(always)]
  pub fn last_mut(&mut self) -> Option<&mut T> {
    if self.is_empty() {
      None
    } else {
      Some(unsafe { self.get_unchecked_mut(self.length - 1) })
    }
  }

  /// A crate-local unchecked version of `remove`, currently only used in the implementation of
  /// `StaticVecSplice`.
  #[inline]
  pub(crate) fn remove_unchecked(&mut self, index: usize) -> T {
    let old_length = self.length;
    unsafe {
      let self_ptr = self.mut_ptr_at_unchecked(index);
      let res = self_ptr.read();
      self_ptr.offset(1).copy_to(self_ptr, old_length - index - 1);
      self.set_len(old_length - 1);
      res
    }
  }

  /// Asserts that `index` is less than the current length of the StaticVec,
  /// and if so removes the value at that position and returns it. Any values
  /// that exist in later positions are shifted to the left.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(staticvec![1, 2, 3].remove(1), 2);
  /// ```
  #[inline]
  pub fn remove(&mut self, index: usize) -> T {
    let old_length = self.length;
    assert!(
      index < old_length,
      "Bounds check failure in `StaticVec::remove`!"
    );
    unsafe {
      let self_ptr = self.mut_ptr_at_unchecked(index);
      let res = self_ptr.read();
      self_ptr.offset(1).copy_to(self_ptr, old_length - index - 1);
      self.set_len(old_length - 1);
      res
    }
  }

  /// Removes the first instance of `item` from the StaticVec if the item exists.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(staticvec![1, 2, 2, 3].remove_item(&2), Some(2));
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec!["AAA", "BBB", "CCC", "DDD"];
  /// assert_eq!(v.swap_pop(1).unwrap(), "BBB");
  /// assert_eq!(v, ["AAA", "DDD", "CCC"]);
  /// ```
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec!["AAA", "BBB", "CCC", "DDD"];
  /// assert_eq!(v.swap_remove(1), "BBB");
  /// assert_eq!(v, ["AAA", "DDD", "CCC"]);
  /// ```
  #[inline(always)]
  pub fn swap_remove(&mut self, index: usize) -> T {
    assert!(
      index < self.length,
      "Bounds check failure in `StaticVec::swap_remove`!"
    );
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<i32, 5>::from([1, 2, 3]);
  /// v.insert(1, 4);
  /// assert_eq!(v, [1, 4, 2, 3]);
  /// ```
  #[inline]
  pub fn insert(&mut self, index: usize, value: T) {
    let old_length = self.length;
    assert!(
      old_length < N && index <= old_length,
      "Insufficient remaining capacity or bounds check failure in `StaticVec::insert`!"
    );
    unsafe {
      let self_ptr = self.mut_ptr_at_unchecked(index);
      self_ptr.copy_to(self_ptr.offset(1), old_length - index);
      self_ptr.write(value);
      self.set_len(old_length + 1);
    }
  }

  /// Functionally equivalent to [`insert`](crate::StaticVec::insert), except with multiple
  /// items provided by an iterator as opposed to just one. This function will panic up-front if
  /// `index` is out of bounds or if the StaticVec does not have a sufficient amount of remaining
  /// capacity, but once the iteration has started will just return immediately if / when the
  /// StaticVec reaches maximum capacity, regardless of whether the iterator still has more items
  /// to yield.
  ///
  /// For safety reasons, as StaticVec cannot increase in capacity, the
  /// iterator is required to implement [`ExactSizeIterator`](core::iter::ExactSizeIterator)
  /// rather than just [`Iterator`](core::iter::Iterator) (though this function still does
  /// the appropriate checking internally to avoid dangerous outcomes in the event of a blatantly
  /// incorrect [`ExactSizeIterator`](core::iter::ExactSizeIterator) implementation.)
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<usize, 8>::from([1, 2, 3, 4, 7, 8]);
  /// v.insert_many(4, staticvec![5, 6].into_iter());
  /// assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8]);
  /// ```
  #[inline]
  pub fn insert_many<I: IntoIterator<Item = T>>(&mut self, index: usize, iter: I)
  where I::IntoIter: ExactSizeIterator<Item = T> {
    let old_length = self.length;
    assert!(
      old_length < N && index <= old_length,
      "Insufficient remaining capacity or bounds check failure in `StaticVec::insert_many`!"
    );
    let mut it = iter.into_iter();
    if index == old_length {
      return self.extend(it);
    }
    let iter_size = it.len();
    assert!(
      index + iter_size >= index && (old_length - index) + iter_size < N,
      "Insufficient remaining capacity or bounds check failure in `StaticVec::insert_many`!"
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
      self.set_len(old_length + item_count);
    }
  }

  /// Functionally equivalent to [`insert_many`](crate::StaticVec::insert_many), except with
  /// multiple items provided by a slice reference as opposed to an arbitrary iterator. Locally
  /// requires that `T` implements [`Copy`](core::marker::Copy) to avoid soundness issues.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<usize, 8>::from([1, 2, 3, 4, 7, 8]);
  /// v.insert_from_slice(4, &[5, 6]);
  /// assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8]);
  /// ```
  #[inline]
  pub fn insert_from_slice(&mut self, index: usize, values: &[T])
  where T: Copy {
    let old_length = self.length;
    let values_length = values.len();
    assert!(
      old_length < N && index <= old_length && values_length <= self.remaining_capacity(),
      "Insufficient remaining capacity or bounds check failure in `StaticVec::insert_from_slice`!"
    );
    unsafe {
      let self_ptr = self.mut_ptr_at_unchecked(index);
      self_ptr.copy_to(self_ptr.add(values_length), old_length - index);
      self_ptr.copy_from_nonoverlapping(values.as_ptr(), values_length);
      self.set_len(old_length + values_length);
    }
  }

  /// Inserts `value` at `index` if the current length of the StaticVec is less than `N` and `index`
  /// is less than the length, or returns a [`CapacityError`](crate::errors::CapacityError)
  /// otherwise. Any values that exist in positions after `index` are shifted to the right.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut vec = StaticVec::<i32, 5>::from([1, 2, 3, 4, 5]);
  /// assert_eq!(vec.try_insert(2, 0), Err(CapacityError::<5> {}));
  /// ```
  #[inline]
  pub fn try_insert(&mut self, index: usize, value: T) -> Result<(), CapacityError<N>> {
    let old_length = self.length;
    if old_length < N && index <= old_length {
      unsafe {
        let self_ptr = self.mut_ptr_at_unchecked(index);
        self_ptr.copy_to(self_ptr.offset(1), old_length - index);
        self_ptr.write(value);
        self.set_len(old_length + 1);
      }
      Ok(())
    } else {
      Err(CapacityError {})
    }
  }

  /// Does the same thing as [`insert_from_slice`](crate::StaticVec::insert_from_slice), but returns
  /// a [`CapacityError`](crate::errors::CapacityError) in the event that something goes wrong as
  /// opposed to relying on internal assertions.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v1 = StaticVec::<usize, 8>::from([1, 2, 3, 4, 7, 8]);
  /// assert!(v1.try_insert_from_slice(4, &[5, 6]).is_ok());
  /// assert_eq!(v1, [1, 2, 3, 4, 5, 6, 7, 8]);
  /// let mut v2 = StaticVec::<usize, 8>::from([1, 2, 3, 4, 7, 8]);
  /// assert!(v2.try_insert_from_slice(207, &[5, 6]).is_err());
  /// ```
  #[inline]
  pub fn try_insert_from_slice(
    &mut self,
    index: usize,
    values: &[T],
  ) -> Result<(), CapacityError<N>>
  where
    T: Copy,
  {
    let old_length = self.length;
    let values_length = values.len();
    if old_length < N && index <= old_length && values_length <= self.remaining_capacity() {
      unsafe {
        let self_ptr = self.mut_ptr_at_unchecked(index);
        self_ptr.copy_to(self_ptr.add(values_length), old_length - index);
        self_ptr.copy_from_nonoverlapping(values.as_ptr(), values_length);
        self.set_len(old_length + values_length);
      }
      Ok(())
    } else {
      Err(CapacityError {})
    }
  }

  /// Returns `true` if `value` is present in the StaticVec.
  /// Locally requires that `T` implements [`PartialEq`](core::cmp::PartialEq)
  /// to make it possible to compare the elements of the StaticVec with `value`.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(staticvec![1, 2, 3].contains(&2), true);
  /// assert_eq!(staticvec![1, 2, 3].contains(&4), false);
  /// ```
  #[inline(always)]
  pub fn contains(&self, value: &T) -> bool
  where T: PartialEq {
    self.iter().any(|current| current == value)
  }

  /// Removes all contents from the StaticVec and sets its length back to 0.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 3];
  /// assert_eq!(v.len(), 3);
  /// assert_eq!(v, [1, 2, 3]);
  /// v.clear();
  /// assert_eq!(v.len(), 0);
  /// assert_eq!(v, []);
  /// ```
  #[inline(always)]
  pub fn clear(&mut self) {
    unsafe { ptr::drop_in_place(self.as_mut_slice()) };
    self.length = 0;
  }

  /// Returns a [`StaticVecIterConst`](crate::iterators::StaticVecIterConst) over the StaticVec's
  /// inhabited area.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let v = staticvec![4, 3, 2, 1];
  /// for i in v.iter() {
  ///   println!("{}", i);
  /// }
  /// ```
  #[inline(always)]
  pub fn iter(&self) -> StaticVecIterConst<T, N> {
    let start_ptr = self.as_ptr();
    unsafe {
      // `start_ptr` will never be null, so this is a safe assumption to give the optimizer.
      intrinsics::assume(!start_ptr.is_null());
      StaticVecIterConst {
        start: start_ptr,
        end: match intrinsics::size_of::<T>() {
          0 => (start_ptr as *const u8).wrapping_add(self.length) as *const T,
          _ => start_ptr.add(self.length),
        },
        marker: PhantomData,
      }
    }
  }

  /// Returns a [`StaticVecIterMut`](crate::iterators::StaticVecIterMut) over the StaticVec's
  /// inhabited area.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![4, 3, 2, 1];
  /// for i in v.iter_mut() {
  ///   *i -= 1;
  /// }
  /// assert_eq!(v, [3, 2, 1, 0]);
  /// ```
  #[inline(always)]
  pub fn iter_mut(&mut self) -> StaticVecIterMut<T, N> {
    let start_ptr = self.as_mut_ptr();
    unsafe {
      // `start_ptr` will never be null, so this is a safe assumption to give the optimizer.
      intrinsics::assume(!start_ptr.is_null());
      StaticVecIterMut {
        start: start_ptr,
        end: match intrinsics::size_of::<T>() {
          0 => (start_ptr as *mut u8).wrapping_add(self.length) as *mut T,
          _ => start_ptr.add(self.length),
        },
        marker: PhantomData,
      }
    }
  }

  /// Returns a separate, stable-sorted StaticVec of the contents of the StaticVec's inhabited area
  /// without modifying the original data. Locally requires that `T` implements
  /// [`Copy`](core::marker::Copy) to avoid soundness issues, and [`Ord`](core::cmp::Ord) to make
  /// the sorting possible.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const V: StaticVec<StaticVec<i32, 2>, 2> = staticvec![staticvec![1, 3], staticvec![4, 2]];
  /// assert_eq!(
  ///   V.iter().flatten().collect::<StaticVec<i32, 4>>().sorted(),
  ///   [1, 2, 3, 4]
  /// );
  /// ```
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

  /// Returns a separate, unstable-sorted StaticVec of the contents of the StaticVec's inhabited
  /// area without modifying the original data. Locally requires that `T` implements
  /// [`Copy`](core::marker::Copy) to avoid soundness issues, and [`Ord`](core::cmp::Ord) to make
  /// the sorting possible.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const V: StaticVec<StaticVec<i32, 2>, 2> = staticvec![staticvec![1, 3], staticvec![4, 2]];
  /// assert_eq!(
  ///   V.iter().flatten().collect::<StaticVec<i32, 4>>().sorted_unstable(),
  ///   [1, 2, 3, 4]
  /// );
  /// ```
  #[inline]
  pub fn sorted_unstable(&self) -> Self
  where T: Copy + Ord {
    // StaticVec uses specialization to have an optimized version of `Clone` for copy types.
    let mut res = self.clone();
    res.sort_unstable();
    res
  }

  /// Returns a separate, unstable-quicksorted StaticVec of the contents of the StaticVec's
  /// inhabited area without modifying the original data. Locally requires that `T` implements
  /// [`Copy`](core::marker::Copy) to avoid soundness issues, and
  /// [`PartialOrd`](core::cmp::PartialOrd) to make the sorting possible.
  ///
  /// Unlike [`sorted`](crate::StaticVec::sorted) and
  /// [`sorted_unstable`](crate::StaticVec::sorted_unstable), this function does not make use of
  /// Rust's built-in sorting methods, but instead makes direct use of a comparatively
  /// unsophisticated recursive quicksort algorithm implemented in this crate.
  ///
  /// This has the advantage of only needing to have [`PartialOrd`](core::cmp::PartialOrd) as a
  /// constraint as opposed to [`Ord`](core::cmp::Ord), but is very likely less performant for
  /// most inputs, so if the type you're sorting does derive or implement
  /// [`Ord`](core::cmp::Ord) it's recommended that you use [`sorted`](crate::StaticVec::sorted) or
  /// [`sorted_unstable`](crate::StaticVec::sorted_unstable) instead of this function.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const V: StaticVec<StaticVec<i32, 2>, 2> = staticvec![staticvec![1, 3], staticvec![4, 2]];
  /// assert_eq!(
  ///   V.iter().flatten().collect::<StaticVec<i32, 4>>().quicksorted_unstable(),
  ///   [1, 2, 3, 4]
  /// );
  /// ```
  #[inline]
  pub fn quicksorted_unstable(&self) -> Self
  where T: Copy + PartialOrd {
    let length = self.length;
    if length < 2 {
      // StaticVec uses specialization to have an optimized verson of `Clone` for copy types.
      return self.clone();
    }
    let mut res = Self::new_data_uninit();
    let res_ptr = Self::first_ptr_mut(&mut res);
    // Copy the inhabited part of `self` into the array we'll use for the returned StaticVec.
    unsafe { self.as_ptr().copy_to_nonoverlapping(res_ptr, length) };
    // Sort the array, and then build and return a new StaticVec from it.
    quicksort_internal(res_ptr, 0, (length - 1) as isize);
    Self { data: res, length }
  }

  /// Provides the same sorting functionality as
  /// [`quicksorted_unstable`](crate::StaticVec::quicksorted_unstable) (and has the same trait
  /// bound requirements) but operates in-place on the calling StaticVec instance rather than
  /// returning the sorted data in a new one.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![5.0, 4.0, 3.0, 2.0, 1.0];
  /// v.quicksort_unstable();
  /// assert_eq!(v, [1.0, 2.0, 3.0, 4.0, 5.0]);
  /// // Note that if you are actually sorting floating-point numbers as shown above, and the
  /// // StaticVec contains one or more instances of NAN, the "accuracy" of the sorting will
  /// // essentially be determined by a combination of how many *consecutive* NANs there are,
  /// // as well as how "mixed up" the surrounding valid numbers were to begin with. In any case,
  /// // the outcome of this particular hypothetical scenario will never be any worse than the
  /// // values simply not being sorted quite as you'd hoped.
  /// ```
  #[inline]
  pub fn quicksort_unstable(&mut self)
  where T: Copy + PartialOrd {
    let length = self.length;
    if length < 2 {
      return;
    }
    let self_ptr = self.as_mut_ptr();
    // We know self_ptr will never be null, so this is a safe hint to give the optimizer.
    unsafe { intrinsics::assume(!self_ptr.is_null()) };
    quicksort_internal(self_ptr, 0, (length - 1) as isize);
  }

  /// Returns a separate, reversed StaticVec of the contents of the StaticVec's inhabited area
  /// without modifying the original data. Locally requires that `T` implements
  /// [`Copy`](core::marker::Copy) to avoid soundness issues.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// assert_eq!(staticvec![1, 2, 3].reversed(), [3, 2, 1]);
  /// ```
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
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticVec;
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
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticVec;
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<i32, 8>::new();
  /// v.extend_from_slice(&[1, 2, 3, 4]);
  /// v.extend_from_slice(&[5, 6, 7, 8, 9, 10, 11]);
  /// assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8]);
  /// ```
  #[inline(always)]
  pub fn extend_from_slice(&mut self, values: &[T])
  where T: Copy {
    let old_length = self.length;
    let added_length = values.len().min(N - old_length);
    // Safety: added_length is <= our remaining capacity and values.len.
    unsafe {
      values
        .as_ptr()
        .copy_to_nonoverlapping(self.mut_ptr_at_unchecked(old_length), added_length);
      self.set_len(old_length + added_length);
    }
  }

  /// Copies and appends all elements, if any, of a slice to the StaticVec if the
  /// StaticVec's remaining capacity is greater than the length of the slice, or returns
  /// a [`CapacityError`](crate::errors::CapacityError) otherwise.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = StaticVec::<i32, 8>::new();
  /// assert!(v.try_extend_from_slice(&[1, 2, 3, 4]).is_ok());
  /// assert!(v.try_extend_from_slice(&[5, 6, 7, 8, 9, 10, 11]).is_err());
  /// assert_eq!(v, [1, 2, 3, 4]);
  /// ```
  #[inline(always)]
  pub fn try_extend_from_slice(&mut self, values: &[T]) -> Result<(), CapacityError<N>>
  where T: Copy {
    let old_length = self.length;
    let added_length = values.len();
    if N - old_length < added_length {
      return Err(CapacityError {});
    }
    unsafe {
      values
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut a = StaticVec::<i32, 8>::from([1, 2, 3, 4]);
  /// let mut b = staticvec![1, 2, 3, 4, 5, 6, 7, 8];
  /// a.append(&mut b);
  /// assert_eq!(a.len(), 8);
  /// assert_eq!(a, [1, 2, 3, 4, 1, 2, 3, 4]);
  /// assert_eq!(b, [5, 6, 7, 8]);
  /// ```
  #[inline]
  pub fn append<const N2: usize>(&mut self, other: &mut StaticVec<T, N2>) {
    let old_length = self.length;
    // Get the maximum number of items both within our capacity and within
    // what `other` actually has to offer.
    let item_count = (N - old_length).min(other.length);
    // Calculate what the length of `other` should be changed to once we've
    // moved the items from it into self.
    let other_new_length = other.length - item_count;
    unsafe {
      // Copy over the items.
      self
        .mut_ptr_at_unchecked(old_length)
        .copy_from_nonoverlapping(other.as_ptr(), item_count);
      // Shift the items leftwards in `other` if / as necessary. This only
      // really does anything if it's the case that the remaining capacity
      // of `self` was less than the number of items `other` had available.
      other
        .as_mut_ptr()
        .copy_from(other.ptr_at_unchecked(item_count), other_new_length);
      // Adjust the lengths of `other` and `self`.
      other.set_len(other_new_length);
      self.set_len(old_length + item_count);
    }
  }

  /*
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
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
  /// assert!(staticvec!['a', 'b'].concat(&staticvec!['c', 'd']) == ['a', 'b', 'c', 'd']);
  /// ```
  #[inline]
  pub fn concat<const N2: usize>(&self, other: &StaticVec<T, N2>) -> StaticVec<T, { N + N2 }>
  where T: Copy {
    let length = self.length;
    let other_length = other.length;
    let mut res = StaticVec::new();
    unsafe {
      // Copy over all of `self`.
      self
        .as_ptr()
        .copy_to_nonoverlapping(res.as_mut_ptr(), length);
      // Copy over all of `other` starting at the position immediately following
      // the last occupied position of the copy we just did from `self`.
      other
        .as_ptr()
        .copy_to_nonoverlapping(res.mut_ptr_at_unchecked(length), other_length);
      // Set the length of the resulting StaticVec before we return it.
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
  /// assert!(staticvec!["a", "b"].concat_clone(&staticvec!["c", "d"]) == ["a", "b", "c", "d"]);
  /// ```
  #[inline]
  pub fn concat_clone<const N2: usize>(
    &self,
    other: &StaticVec<T, N2>,
  ) -> StaticVec<T, { N + N2 }>
  where
    T: Clone,
  {
    let mut res = StaticVec::new();
    for item in self {
      unsafe { res.push_unchecked(item.clone()) };
    }
    for item in other {
      unsafe { res.push_unchecked(item.clone()) };
    }
    res
  }
  */

  /*
  /// Returns a new StaticVec consisting of the elements of `self` in linear order, interspersed
  /// with a copy of `separator` between each one.
  ///
  /// Locally requires that `T` implements [`Copy`](core::marker::Copy) to
  /// avoid soundness issues and also allow for a more efficient implementation than would otherwise
  /// be possible.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
  /// assert_eq!(
  ///  staticvec!["A", "B", "C", "D"].intersperse("Z"),
  ///  ["A", "Z", "B", "Z", "C", "Z", "D"]
  /// );
  /// ```
  #[inline]
  pub fn intersperse(&self, separator: T) -> StaticVec<T, { N * 2 }>
  where T: Copy {
    if self.is_empty() {
      return StaticVec::new();
    }
    let mut res = StaticVec::new();
    // The `as *mut T` cast below is necessary to make the type inference work properly (at the
    // moment at least). `rustc` still gets a bit confused by math operations done on const generic
    // values in return types it seems.
    // Note that the `StaticVec::new()` calls above *have* to be written without any constraints,
    // as otherwise we'll hit a particular bug where `rustc` says:
    // "expected struct `StaticVec<_, { N * 2 }>`, found struct `StaticVec<_, { N * 2 }>`".
    let mut res_ptr = res.as_mut_ptr() as *mut T;
    let mut i = 0;
    let length = self.length;
    while i < length - 1 {
      unsafe {
        res_ptr.write(self.ptr_at_unchecked(i).read());
        res_ptr.offset(1).write(separator);
        res_ptr = res_ptr.offset(2);
      }
      i += 1;
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
  /// assert_eq!(
  ///  staticvec!["A", "B", "C", "D"].intersperse_clone("Z"),
  ///  ["A", "Z", "B", "Z", "C", "Z", "D"]
  /// );
  /// ```
  #[inline]
  pub fn intersperse_clone(&self, separator: T) -> StaticVec<T, { N * 2 }>
  where T: Clone {
    if self.is_empty() {
      return StaticVec::new();
    }
    let mut res = StaticVec::new();
    let length = self.length;
    unsafe {
      for i in 0..length - 1 {
        res.push_unchecked(self.get_unchecked(i).clone());
        res.push_unchecked(separator.clone());
      }
      res.push_unchecked(self.get_unchecked(length - 1).clone());
    }
    res
  }
  */

  /// Returns a StaticVec containing the contents of a [`Vec`](alloc::vec::Vec) instance.
  /// If the [`Vec`](alloc::vec::Vec) has a length greater than the declared capacity of the
  /// resulting StaticVec, any contents after that point are ignored. Note that using this function
  /// consumes the source [`Vec`](alloc::vec::Vec).
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = vec![1, 2, 3];
  /// let sv: StaticVec<i32, 3> = StaticVec::from_vec(v);
  /// assert_eq!(sv, [1, 2, 3]);
  /// ```
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline]
  pub fn from_vec(mut vec: Vec<T>) -> Self {
    let vec_len = vec.len();
    let item_count = vec_len.min(N);
    Self {
      data: {
        // Set the length of `vec` to 0 to prevent double-drops.
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut sv = staticvec![1, 2, 3];
  /// let v = StaticVec::into_vec(sv);
  /// assert_eq!(v, [1, 2, 3]);
  /// ```
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
      // Set the length of `self` to 0 to prevent double-drops.
      self.set_len(0);
      res
    }
  }

  /// Inspired by the function of the same name from `ArrayVec`, this function directly returns
  /// the StaticVec's backing array (as a "normal" array not wrapped in an instance of
  /// `MaybeUninit`) in `Ok` if and only if the StaticVec is at maximum capacity. Otherwise, the
  /// StaticVec itself is returned in `Err`.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v1 = StaticVec::<i32, 4>::new();
  /// v1.push(1);
  /// v1.push(2);
  /// let a = v1.into_inner();
  /// assert!(a.is_err());
  /// let v2 = staticvec![1, 2, 3, 4];
  /// let a = v2.into_inner();
  /// assert!(a.is_ok());
  /// assert_eq!(a.unwrap(), [1, 2, 3, 4]);
  /// ```
  #[inline(always)]
  pub fn into_inner(mut self) -> Result<[T; N], Self> {
    if self.is_not_full() {
      Err(self)
    } else {
      // Set the length of `self` to 0 to prevent double-drops.
      self.length = 0;
      // Read out the contents of `data`.
      unsafe { Ok(self.data.assume_init_read()) }
    }
  }

  /// Removes the specified range of elements from the StaticVec and returns them in a new one.
  ///
  /// # Panics
  ///
  /// Panics if the range's starting point is greater than the end point or if the end point is
  /// greater than the length of the StaticVec.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 3];
  /// let u = v.drain(1..);
  /// assert_eq!(v, &[1]);
  /// ```
  #[inline]
  pub fn drain<R>(&mut self, range: R) -> Self
  // No Copy bounds here because the original StaticVec gives up all access to the values in
  // question.
  where R: RangeBounds<usize> {
    // Borrowed this part from normal Vec's implementation.
    let old_length = self.length;
    let start = match range.start_bound() {
      Included(&idx) => idx,
      Excluded(&idx) => idx + 1,
      Unbounded => 0,
    };
    let end = match range.end_bound() {
      Included(&idx) => idx + 1,
      Excluded(&idx) => idx,
      Unbounded => old_length,
    };
    assert!(
      start <= end && end <= old_length,
      "Bounds check failure in `StaticVec::drain`!"
    );
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
            .copy_to(self.mut_ptr_at_unchecked(start), old_length - end);
          self.set_len(old_length - res_length);
          res
        }
      },
      length: res_length,
    }
  }

  /// Removes the specified range of elements from the StaticVec and returns them in a
  /// [`StaticVecDrain`](crate::iterators::StaticVecDrain).
  ///
  /// # Panics
  ///
  /// Panics if the range's starting point is greater than the end point or if the end point is
  /// greater than the length of the StaticVec.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v1 = staticvec![0, 4, 5, 6, 7];
  /// let v2: StaticVec<i32, 3> = v1.drain_iter(1..4).rev().collect();
  /// assert_eq!(v2, [6, 5, 4]);
  /// ```
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
    assert!(
      start <= end && end <= length,
      "Bounds check failure in `StaticVec::drain_iter`!"
    );
    unsafe {
      // Set the length to `start` to avoid memory issues if anything goes wrong with the Drain.
      self.set_len(start);
      let start_ptr = self.ptr_at_unchecked(start);
      // `start_ptr` will never be null, so this is a safe assumption to give to
      // the optimizer.
      intrinsics::assume(!start_ptr.is_null());
      // Create the StaticVecDrain from the specified range.
      StaticVecDrain {
        start: end,
        length: length - end,
        iter: StaticVecIterConst {
          start: start_ptr,
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

  /// Removes all elements in the StaticVec for which `filter` returns true and returns them in a
  /// new one.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut numbers = staticvec![1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15];
  /// let evens = numbers.drain_filter(|x| *x % 2 == 0);
  /// let odds = numbers;
  /// assert_eq!(evens, [2, 4, 6, 8, 14]);
  /// assert_eq!(odds, [1, 3, 5, 9, 11, 13, 15]);
  /// ```
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

  /// Replaces the specified range in the StaticVec with the contents of `replace_with` and returns
  /// the removed items in an instance of [`StaticVecSplice`](crate::iterators::StaticVecSplice).
  /// `replace_with` does not need to be the same length as `range`. Returns immediately if and when
  /// the StaticVec reaches maximum capacity, regardless of whether or not `replace_with` still has
  /// more items to yield.
  ///
  /// # Panics
  ///
  /// Panics if the range's starting point is greater than the end point or if the end point is
  /// greater than the length of the StaticVec.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 3];
  /// let new = [7, 8];
  /// let u: StaticVec<u8, 2> = v.splice(..2, new.iter().copied()).collect();
  /// assert_eq!(v, [7, 8, 3]);
  /// assert_eq!(u, [1, 2]);
  /// ```
  #[inline]
  pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> StaticVecSplice<T, I::IntoIter, N>
  where
    R: RangeBounds<usize>,
    I: IntoIterator<Item = T>, {
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
    assert!(
      start <= end && end <= length,
      "Bounds check failure in `StaticVec::splice`!"
    );
    StaticVecSplice {
      start,
      end,
      vec: self,
      replace_with: replace_with.into_iter(),
    }
  }

  /// Removes all elements in the StaticVec for which `filter` returns false.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 3, 4, 5];
  /// let keep = staticvec![false, true, true, false, true];
  /// let mut i = 0;
  /// v.retain(|_| (keep[i], i += 1).0);
  /// assert_eq!(v, [2, 3, 5]);
  /// ```
  #[inline(always)]
  pub fn retain<F>(&mut self, mut filter: F)
  where F: FnMut(&T) -> bool {
    self.drain_filter(|val| !filter(val));
  }

  /// Shortens the StaticVec, keeping the first `length` elements and dropping the rest.
  /// Does nothing if `length` is greater than or equal to the current length of the StaticVec.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 3, 4, 5];
  /// v.truncate(2);
  /// assert_eq!(v, [1, 2]);
  /// ```
  #[inline(always)]
  pub fn truncate(&mut self, length: usize) {
    if length < self.length {
      let old_length = self.length;
      unsafe {
        self.set_len(length);
        ptr::drop_in_place(slice_from_raw_parts_mut(
          self.mut_ptr_at_unchecked(length),
          old_length - length,
        ));
      }
    }
  }

  /// Splits the StaticVec into two at the given index. The original StaticVec will contain elements
  /// `0..at`, and the new one will contain elements `at..self.len()`.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v1 = staticvec![1, 2, 3];
  /// let v2 = v1.split_off(1);
  /// assert_eq!(v1, [1]);
  /// assert_eq!(v2, [2, 3]);
  /// ```
  #[inline]
  pub fn split_off(&mut self, at: usize) -> Self {
    let old_length = self.length;
    assert!(
      at <= old_length,
      "Bounds check failure in `StaticVec::split_off`!"
    );
    let split_length = old_length - at;
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
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec!["aaa", "bbb", "BBB", "ccc", "ddd"];
  /// v.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
  /// assert_eq!(v, ["aaa", "bbb", "ccc", "ddd"]);
  /// ```
  #[inline(always)]
  pub fn dedup_by<F>(&mut self, same_bucket: F)
  where F: FnMut(&mut T, &mut T) -> bool {
    // Mostly the same as Vec's version.
    let new_length = self.as_mut_slice().partition_dedup_by(same_bucket).0.len();
    self.truncate(new_length);
  }

  /// Removes consecutive repeated elements in the StaticVec according to the
  /// locally required [`PartialEq`](core::cmp::PartialEq) trait implementation for `T`.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![1, 2, 2, 3, 2];
  /// v.dedup();
  /// assert_eq!(v, [1, 2, 3, 2]);
  /// ```
  #[inline(always)]
  pub fn dedup(&mut self)
  where T: PartialEq {
    // Exactly the same as Vec's version.
    self.dedup_by(|a, b| a == b)
  }

  /// Removes all but the first of consecutive elements in the StaticVec that
  /// resolve to the same key.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// let mut v = staticvec![10, 20, 21, 30, 20];
  /// v.dedup_by_key(|i| *i / 10);
  /// assert_eq!(v, [10, 20, 30, 20]);
  /// ```
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
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
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

  /*
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
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
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
  */

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
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
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

  /*
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
  /// # Example usage:
  /// ```
  /// # use staticvec::staticvec;
  /// assert_eq!(
  ///   staticvec![1, 2, 3].union(&staticvec![4, 2, 3, 4]),
  ///   [1, 2, 3, 4],
  /// );
  /// ```
  #[inline]
  #[rustfmt::skip]
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
  */

  /// A concept borrowed from the widely-used `SmallVec` crate, this function
  /// returns a tuple consisting of a constant pointer to the first element of the StaticVec,
  /// the length of the StaticVec, and the capacity of the StaticVec.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::*;
  /// static V: StaticVec<usize, 4> = staticvec![4, 5, 6, 7];
  /// assert_eq!(V.triple(), (V.as_ptr(), 4, 4));
  /// ```
  #[inline(always)]
  pub const fn triple(&self) -> (*const T, usize, usize) {
    (self.as_ptr(), self.length, N)
  }

  /// A mutable version of [`triple`](crate::StaticVec::triple). This implementation differs from
  /// the one found in `SmallVec` in that it only provides the first element of the StaticVec as
  /// a mutable pointer, not also the length as a mutable reference.
  ///
  /// Example:
  /// ```
  /// # use::staticvec::*;
  /// let mut v = staticvec![4, 5, 6, 7];
  /// let t = v.triple_mut();
  /// assert_eq!(t, (v.as_mut_ptr(), 4, 4));
  /// unsafe { *t.0 = 8 };
  /// assert_eq!(v, [8, 5, 6, 7]);
  /// ```
  #[inline(always)]
  pub const fn triple_mut(&mut self) -> (*mut T, usize, usize) {
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
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.added(&B), [6.0, 8.0, 10.0, 12.0]);
  /// ```
  #[inline(always)]
  pub fn added(&self, other: &Self) -> Self
  where T: Copy + Add<Output = T> {
    assert!(
      self.is_full() && other.is_full(),
      "In `StaticVec::added`, both `self` and `other` must be at maximum capacity!"
    );
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
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.subtracted(&B), [2.0, 2.0, 2.0, 2.0]);
  /// ```
  #[inline(always)]
  pub fn subtracted(&self, other: &Self) -> Self
  where T: Copy + Sub<Output = T> {
    assert!(
      self.is_full() && other.is_full(),
      "In `StaticVec::subtracted`, both `self` and `other` must be at maximum capacity!"
    );
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
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.multiplied(&B), [8.0, 15.0, 24.0, 35.0]);
  /// ```
  #[inline(always)]
  pub fn multiplied(&self, other: &Self) -> Self
  where T: Copy + Mul<Output = T> {
    assert!(
      self.is_full() && other.is_full(),
      "In `StaticVec::multiplied`, both `self` and `other` must be at maximum capacity!"
    );
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
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticvec, StaticVec};
  /// const A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  /// const B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  /// assert_eq!(A.divided(&B), [2.0, 1.6666666666666667, 1.5, 1.4]);
  /// ```
  #[inline(always)]
  pub fn divided(&self, other: &Self) -> Self
  where T: Copy + Div<Output = T> {
    assert!(
      self.is_full() && other.is_full(),
      "In `StaticVec::divided`, both `self` and `other` must be at maximum capacity!"
    );
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

impl<const N: usize> StaticVec<u8, N> {
  /// Called solely in `__new_from_const_str`, where the input `MaybeUninit` is guaranteed to have
  /// been properly initialized starting at the beginning with the bytes of an `&str` literal,
  /// and the input `length` is the known-at-compile-time length of said literal.
  #[doc(hidden)]
  #[inline(always)]
  pub(crate) const fn new_from_str_data(data: MaybeUninit<[u8; N]>, length: usize) -> Self {
    Self { data, length }
  }

  /// Called solely in `__new_from_const_str`, where `values` is guaranteed to be the slice
  /// representation of a proper `&str` literal.
  #[doc(hidden)]
  #[inline]
  pub(crate) const fn bytes_to_data(values: &[u8]) -> MaybeUninit<[u8; N]> {
    // What follows is an idea partially arrived at from reading the source of the `const-concat`
    // crate. Note that it amounts to effectively a `const fn` compatible implementation of what
    // `MaybeUninit::assume_uninit()` does, and is *only* used here due to there being no other way
    // to get an instance of `[MaybeUninit<u8>; N]` that we can actually write to (and to be clear,
    // *not* read from) using regular indexing in conjunction with the `const_loop` feature (which
    // is itself the only way at this time to write an arbitrary number of bytes from `values` to
    // the result array at compile time).
    #[repr(C)]
    union Convert<From: Copy, To: Copy> {
      from: From,
      to: To,
    }
    // As stated above, this is effectively doing what `MaybeUninit::assume_init()` does.
    // Note that while it might "look scary", what this function actually does would be incredibly
    // mundane in basically any other language: you would just declare a very normal static array,
    // and use it, very normally. That's literally *all* this is.
    let mut res = unsafe {
      Convert::<MaybeUninit<[MaybeUninit<u8>; N]>, [MaybeUninit<u8>; N]> {
        from: MaybeUninit::uninit(),
      }
      .to
    };
    // Move `values.len()` worth of bytes from `values` to `res`. I'm unaware of any other way that
    // this could be done currently that would leave us with something usable to create a StaticVec
    // for which the generic `N` could be *different* from `values.len()`, so thank
    // you, `const_loop`!
    let mut i = 0;
    while i < values.len() {
      // We've statically asserted that `values.len() <= N` before entering this overall function,
      // so there's no concern that we might go out of bounds here (although that would still just
      // result in compilation not actually succeeding at all due to the `const` index error).
      res[i] = MaybeUninit::new(values[i]);
      i += 1;
    }
    // Convert `res` from an instance of `[MaybeUninit<u8>; N]` to one of `[u8; N]`, and then return
    // it as an instance of `MaybeUninit<[u8; N]>` that can be used to construct a `StaticVec`.
    MaybeUninit::new(unsafe { Convert::<[MaybeUninit<u8>; N], [u8; N]> { from: res }.to })
  }

  /// Called solely from inside the `staticstring!` macro, and so must be public. This is guaranteed
  /// to return a correctly initialized `StaticVec<u8, N>`, but we give it the two-underscore
  /// prefix and hide it from `rustdoc` anyways just so no one thinks it's for general use.
  #[doc(hidden)]
  #[inline(always)]
  pub const fn __new_from_const_str(values: &str) -> Self {
    // This works at compile time too, of course, thanks to the `const_panic` feature.
    assert!(
      values.len() <= N,
      // At the moment, I don't think this message is actually printed in any context when the
      // assertion gets triggered (currently it's just "could not evaluate static initializer") but
      // I feel like it doesn't hurt to have here just in case the compiler-error behavior changes
      // such that custom messages are actually shown.
      "Attempted to create a `StaticString` with insufficient capacity from an `&str` literal!"
    );
    Self::new_from_str_data(Self::bytes_to_data(values.as_bytes()), values.len())
  }
}
