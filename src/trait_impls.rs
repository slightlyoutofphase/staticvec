use core::borrow::{Borrow, BorrowMut};
use core::cmp::{Eq, Ord, Ordering, PartialEq};
use core::fmt::{self, Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::mem::MaybeUninit;
use core::ops::{
  Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
  RangeToInclusive,
};
use core::ptr;

use crate::heap::StaticHeap;
use crate::iterators::*;
use crate::string::StaticString;
use crate::utils::{partial_compare, slice_from_raw_parts, slice_from_raw_parts_mut};
use crate::StaticVec;

#[cfg(feature = "std")]
use core::str;

#[cfg(feature = "std")]
use alloc::string::String;

#[cfg(feature = "std")]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::io::{self, BufRead, IoSlice, IoSliceMut, Read, Write};

#[cfg(feature = "serde_support")]
use core::marker::PhantomData;

#[cfg(feature = "serde_support")]
use serde::{
  de::{SeqAccess, Visitor},
  Deserialize, Deserializer, Serialize, Serializer,
};

impl<T, const N: usize> AsMut<[T]> for StaticVec<T, N> {
  #[inline(always)]
  fn as_mut(&mut self) -> &mut [T] {
    self.as_mut_slice()
  }
}

impl<T, const N: usize> AsRef<[T]> for StaticVec<T, N> {
  #[inline(always)]
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<T, const N: usize> Borrow<[T]> for StaticVec<T, N> {
  #[inline(always)]
  fn borrow(&self) -> &[T] {
    &self[..]
  }
}

impl<T, const N: usize> BorrowMut<[T]> for StaticVec<T, N> {
  #[inline(always)]
  fn borrow_mut(&mut self) -> &mut [T] {
    &mut self[..]
  }
}

impl<T: Clone, const N: usize> Clone for StaticVec<T, N> {
  #[inline]
  default fn clone(&self) -> Self {
    let mut res = Self::new();
    for item in self {
      // Safety: `self` has the same capacity of `res`, and `res` is
      // empty, so all of these pushes are safe.
      unsafe {
        res.push_unchecked(item.clone());
      }
    }
    res
  }

  #[inline]
  default fn clone_from(&mut self, other: &Self) {
    let other_length = other.length;
    self.truncate(other_length);
    let self_length = self.length;
    for i in 0..self_length {
      // Safety: after the truncate, `self.len` <= `other.len`, which means that for
      // every `i` in `self`, there is definitely an element at `other[i]`.
      unsafe {
        self.get_unchecked_mut(i).clone_from(other.get_unchecked(i));
      }
    }
    for i in self_length..other_length {
      // Safety: `i` < `other.length`, so `other.get_unchecked` is safe. `i` starts at
      // `self.length`, which is <= `other.length`, so there is always an available
      // slot at `self[i]` to push into.
      unsafe {
        self.push_unchecked(other.get_unchecked(i).clone());
      }
    }
  }
}

impl<T: Copy, const N: usize> Clone for StaticVec<T, N> {
  #[inline(always)]
  fn clone(&self) -> Self {
    let length = self.length;
    match length {
      // If `self` is empty, just return a new StaticVec.
      0 => Self::new(),
      _ => Self {
        data: {
          let mut res = Self::new_data_uninit();
          unsafe {
            self
              .as_ptr()
              .copy_to_nonoverlapping(Self::first_ptr_mut(&mut res), length);
            res
          }
        },
        length,
      },
    }
  }

  #[inline(always)]
  fn clone_from(&mut self, rhs: &Self) {
    // Here we take advantage of the above efficient `clone` implementation,
    // in reverse.
    *self = rhs.clone();
  }
}

impl<T: Debug, const N: usize> Debug for StaticVec<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_list().entries(self.as_slice()).finish()
  }
}

impl<T, const N: usize> Default for StaticVec<T, N> {
  /// Calls `new`.
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const N: usize> Deref for StaticVec<T, N> {
  type Target = [T];
  #[inline(always)]
  fn deref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<T, const N: usize> DerefMut for StaticVec<T, N> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut [T] {
    self.as_mut_slice()
  }
}

impl<T, const N: usize> Drop for StaticVec<T, N> {
  #[inline(always)]
  fn drop(&mut self) {
    unsafe { ptr::drop_in_place(self.as_mut_slice()) };
  }
}

impl<T: Eq, const N: usize> Eq for StaticVec<T, N> {}

/// A helper trait for specialization-based implementations of [`Extend`](core::iter::Extend) and
/// ['FromIterator`](core::iter::FromIterator).
pub(crate) trait ExtendEx<T, I> {
  fn extend_ex(&mut self, iter: I);
  fn from_iter_ex(iter: I) -> Self;
}

impl<T, I: IntoIterator<Item = T>, const N: usize> ExtendEx<T, I> for StaticVec<T, N> {
  impl_extend_ex!(val, val);
  impl_from_iter_ex!(val, val);
}

impl<'a, T: 'a + Copy, I: IntoIterator<Item = &'a T>, const N: usize> ExtendEx<&'a T, I>
  for StaticVec<T, N>
{
  impl_extend_ex!(val, (*val));
  impl_from_iter_ex!(val, (*val));
}

impl<T, const N1: usize, const N2: usize> ExtendEx<T, StaticVec<T, N1>> for StaticVec<T, N2> {
  #[inline(always)]
  default fn extend_ex(&mut self, ref mut iter: StaticVec<T, N1>) {
    self.append(iter);
  }

  #[inline]
  default fn from_iter_ex(iter: StaticVec<T, N1>) -> Self {
    Self {
      data: {
        unsafe {
          let mut data = Self::new_data_uninit();
          iter
            .as_ptr()
            .copy_to_nonoverlapping(Self::first_ptr_mut(&mut data), N1.min(N2));
          // We use the same sort of sequence here as in `new_from_array`.
          if N1 != N2 {
            let mut forgotten = MaybeUninit::new(iter);
            ptr::drop_in_place(
              forgotten
                .get_mut()
                .as_mut_slice()
                .get_unchecked_mut(N1.min(N2)..N1),
            );
          }
          data
        }
      },
      length: N1.min(N2),
    }
  }
}

impl<'a, T: 'a + Copy, const N1: usize, const N2: usize> ExtendEx<&'a T, &StaticVec<T, N2>>
  for StaticVec<T, N1>
{
  #[inline(always)]
  default fn extend_ex(&mut self, iter: &StaticVec<T, N2>) {
    self.extend_from_slice(iter);
  }

  #[inline(always)]
  default fn from_iter_ex(iter: &StaticVec<T, N2>) -> Self {
    Self::new_from_slice(iter)
  }
}

impl<'a, T: 'a + Copy, const N: usize> ExtendEx<&'a T, &StaticVec<T, N>> for StaticVec<T, N> {
  #[inline(always)]
  fn extend_ex(&mut self, iter: &StaticVec<T, N>) {
    self.extend_from_slice(iter);
  }

  #[inline(always)]
  fn from_iter_ex(iter: &StaticVec<T, N>) -> Self {
    Self::new_from_slice(iter)
  }
}

impl<'a, T: 'a + Copy, const N1: usize, const N2: usize>
  ExtendEx<&'a T, StaticVecIterConst<'a, T, N2>> for StaticVec<T, N1>
{
  #[inline(always)]
  default fn extend_ex(&mut self, iter: StaticVecIterConst<'a, T, N2>) {
    self.extend_from_slice(iter.as_slice());
  }

  #[inline(always)]
  default fn from_iter_ex(iter: StaticVecIterConst<'a, T, N2>) -> Self {
    Self::new_from_slice(iter.as_slice())
  }
}

impl<'a, T: 'a + Copy, const N: usize> ExtendEx<&'a T, StaticVecIterConst<'a, T, N>>
  for StaticVec<T, N>
{
  #[inline(always)]
  fn extend_ex(&mut self, iter: StaticVecIterConst<'a, T, N>) {
    self.extend_from_slice(iter.as_slice());
  }

  #[inline(always)]
  fn from_iter_ex(iter: StaticVecIterConst<'a, T, N>) -> Self {
    Self::new_from_slice(iter.as_slice())
  }
}

impl<'a, T: 'a + Copy, const N: usize> ExtendEx<&'a T, core::slice::Iter<'a, T>>
  for StaticVec<T, N>
{
  #[inline(always)]
  fn extend_ex(&mut self, iter: core::slice::Iter<'a, T>) {
    self.extend_from_slice(iter.as_slice());
  }

  #[inline(always)]
  fn from_iter_ex(iter: core::slice::Iter<'a, T>) -> Self {
    Self::new_from_slice(iter.as_slice())
  }
}

impl<'a, T: 'a + Copy, const N: usize> ExtendEx<&'a T, &'a [T]> for StaticVec<T, N> {
  #[inline(always)]
  fn extend_ex(&mut self, iter: &'a [T]) {
    self.extend_from_slice(iter);
  }

  #[inline(always)]
  fn from_iter_ex(iter: &'a [T]) -> Self {
    Self::new_from_slice(iter)
  }
}

impl<T, const N: usize> Extend<T> for StaticVec<T, N> {
  #[inline(always)]
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    <Self as ExtendEx<T, I>>::extend_ex(self, iter);
  }
}

impl<'a, T: 'a + Copy, const N: usize> Extend<&'a T> for StaticVec<T, N> {
  #[inline(always)]
  fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
    <Self as ExtendEx<&'a T, I>>::extend_ex(self, iter);
  }
}

impl<T: Copy, const N: usize> From<&[T]> for StaticVec<T, N> {
  /// Creates a new StaticVec instance from the contents of `values`, using
  /// [`new_from_slice`](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &[T]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T: Copy, const N: usize> From<&mut [T]> for StaticVec<T, N> {
  /// Creates a new StaticVec instance from the contents of `values`, using
  /// [`new_from_slice`](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &mut [T]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T, const N1: usize, const N2: usize> From<[T; N1]> for StaticVec<T, N2> {
  /// Creates a new StaticVec instance from the contents of `values`, using
  /// [`new_from_array`](crate::StaticVec::new_from_array) internally.
  #[inline(always)]
  default fn from(values: [T; N1]) -> Self {
    Self::new_from_array(values)
  }
}

impl<T, const N: usize> From<[T; N]> for StaticVec<T, N> {
  #[inline(always)]
  fn from(values: [T; N]) -> Self {
    Self::new_from_const_array(values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<&[T; N1]> for StaticVec<T, N2> {
  /// Creates a new StaticVec instance from the contents of `values`, using
  /// [`new_from_slice`](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  default fn from(values: &[T; N1]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T: Copy, const N: usize> From<&[T; N]> for StaticVec<T, N> {
  /// Creates a new StaticVec instance from the contents of `values`, using
  /// [`new_from_slice`](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &[T; N]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<&mut [T; N1]> for StaticVec<T, N2> {
  /// Creates a new StaticVec instance from the contents of `values`, using
  /// [`new_from_slice`](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  default fn from(values: &mut [T; N1]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T: Copy, const N: usize> From<&mut [T; N]> for StaticVec<T, N> {
  /// Creates a new StaticVec instance from the contents of `values`, using
  /// [`new_from_slice`](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &mut [T; N]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T, const N1: usize, const N2: usize> From<StaticHeap<T, N1>> for StaticVec<T, N2> {
  #[inline(always)]
  default fn from(heap: StaticHeap<T, N1>) -> StaticVec<T, N2> {
    StaticVec::from_iter(heap.data)
  }
}

impl<T, const N: usize> From<StaticHeap<T, N>> for StaticVec<T, N> {
  #[inline(always)]
  fn from(heap: StaticHeap<T, N>) -> StaticVec<T, N> {
    heap.data
  }
}

impl<const N1: usize, const N2: usize> From<StaticString<N1>> for StaticVec<u8, N2> {
  #[inline(always)]
  default fn from(string: StaticString<N1>) -> Self {
    Self::new_from_slice(string.as_bytes())
  }
}

impl<const N: usize> From<StaticString<N>> for StaticVec<u8, N> {
  #[inline(always)]
  fn from(string: StaticString<N>) -> Self {
    string.into_bytes()
  }
}

#[cfg(feature = "std")]
#[doc(cfg(feature = "std"))]
impl<T, const N: usize> From<Vec<T>> for StaticVec<T, N> {
  /// Functionally equivalent to [`from_vec`](crate::StaticVec::from_vec).
  #[inline(always)]
  fn from(vec: Vec<T>) -> Self {
    Self::from_vec(vec)
  }
}

impl<T, const N: usize> FromIterator<T> for StaticVec<T, N> {
  #[inline(always)]
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    <Self as ExtendEx<T, I>>::from_iter_ex(iter)
  }
}

impl<'a, T: 'a + Copy, const N: usize> FromIterator<&'a T> for StaticVec<T, N> {
  #[inline(always)]
  fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
    <Self as ExtendEx<&'a T, I>>::from_iter_ex(iter)
  }
}

impl<T: Hash, const N: usize> Hash for StaticVec<T, N> {
  #[inline(always)]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state);
  }
}

// We implement the various forms of `Index` directly, as after trying out
// deferring to `SliceIndex` for it for a while it proved to to be somewhat
// less performant due to the added indirection.

impl<T, const N: usize> Index<usize> for StaticVec<T, N> {
  type Output = T;
  /// Asserts that `index` is less than the current length of the StaticVec,
  /// and if so returns the value at that position as a constant reference.
  #[inline(always)]
  fn index(&self, index: usize) -> &Self::Output {
    assert!(
      index < self.length,
      "In StaticVec::index, provided index {} must be less than the current length of {}!",
      index,
      self.length
    );
    unsafe { self.get_unchecked(index) }
  }
}

impl<T, const N: usize> IndexMut<usize> for StaticVec<T, N> {
  /// Asserts that `index` is less than the current length of the StaticVec,
  /// and if so returns the value at that position as a mutable reference.
  #[inline(always)]
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    assert!(
      index < self.length,
      "In StaticVec::index_mut, provided index {} must be less than the current length of {}!",
      index,
      self.length
    );
    unsafe { self.get_unchecked_mut(index) }
  }
}

impl<T, const N: usize> Index<Range<usize>> for StaticVec<T, N> {
  type Output = [T];
  /// Asserts that the lower bound of `index` is less than its upper bound,
  /// and that its upper bound is less than or equal to the current length of the StaticVec,
  /// and if so returns a constant reference to a slice of elements `index.start..index.end`.
  #[inline(always)]
  fn index(&self, index: Range<usize>) -> &Self::Output {
    assert!(index.start < index.end && index.end <= self.length);
    slice_from_raw_parts(
      unsafe { self.ptr_at_unchecked(index.start) },
      index.end - index.start,
    )
  }
}

impl<T, const N: usize> IndexMut<Range<usize>> for StaticVec<T, N> {
  /// Asserts that the lower bound of `index` is less than its upper bound,
  /// and that its upper bound is less than or equal to the current length of the StaticVec,
  /// and if so returns a mutable reference to a slice of elements `index.start..index.end`.
  #[inline(always)]
  fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
    assert!(index.start < index.end && index.end <= self.length);
    slice_from_raw_parts_mut(
      unsafe { self.mut_ptr_at_unchecked(index.start) },
      index.end - index.start,
    )
  }
}

impl<T, const N: usize> Index<RangeFrom<usize>> for StaticVec<T, N> {
  type Output = [T];
  /// Asserts that the lower bound of `index` is less than or equal to the
  /// current length of the StaticVec, and if so returns a constant reference
  /// to a slice of elements `index.start()..self.length`.
  #[inline(always)]
  fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
    assert!(index.start <= self.length);
    slice_from_raw_parts(
      unsafe { self.ptr_at_unchecked(index.start) },
      self.length - index.start,
    )
  }
}

impl<T, const N: usize> IndexMut<RangeFrom<usize>> for StaticVec<T, N> {
  /// Asserts that the lower bound of `index` is less than or equal to the
  /// current length of the StaticVec, and if so returns a mutable reference
  /// to a slice of elements `index.start()..self.length`.
  #[inline(always)]
  fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
    assert!(index.start <= self.length);
    slice_from_raw_parts_mut(
      unsafe { self.mut_ptr_at_unchecked(index.start) },
      self.length - index.start,
    )
  }
}

impl<T, const N: usize> Index<RangeFull> for StaticVec<T, N> {
  type Output = [T];
  /// Returns a constant reference to a slice consisting of `0..self.length`
  /// elements of the StaticVec, using [as_slice](crate::StaticVec::as_slice) internally.
  #[inline(always)]
  fn index(&self, _index: RangeFull) -> &Self::Output {
    self.as_slice()
  }
}

impl<T, const N: usize> IndexMut<RangeFull> for StaticVec<T, N> {
  /// Returns a mutable reference to a slice consisting of `0..self.length`
  /// elements of the StaticVec, using [as_mut_slice](crate::StaticVec::as_mut_slice) internally.
  #[inline(always)]
  fn index_mut(&mut self, _index: RangeFull) -> &mut Self::Output {
    self.as_mut_slice()
  }
}

impl<T, const N: usize> Index<RangeInclusive<usize>> for StaticVec<T, N> {
  type Output = [T];
  /// Asserts that the lower bound of `index` is less than or equal to its upper bound,
  /// and that its upper bound is less than the current length of the StaticVec,
  /// and if so returns a constant reference to a slice of elements `index.start()..=index.end()`.
  #[inline(always)]
  fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
    assert!(index.start() <= index.end() && *index.end() < self.length);
    slice_from_raw_parts(
      unsafe { self.ptr_at_unchecked(*index.start()) },
      (index.end() + 1) - index.start(),
    )
  }
}

impl<T, const N: usize> IndexMut<RangeInclusive<usize>> for StaticVec<T, N> {
  /// Asserts that the lower bound of `index` is less than or equal to its upper bound,
  /// and that its upper bound is less than the current length of the StaticVec,
  /// and if so returns a mutable reference to a slice of elements `index.start()..=index.end()`.
  #[inline(always)]
  fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
    assert!(index.start() <= index.end() && *index.end() < self.length);
    slice_from_raw_parts_mut(
      unsafe { self.mut_ptr_at_unchecked(*index.start()) },
      (index.end() + 1) - index.start(),
    )
  }
}

impl<T, const N: usize> Index<RangeTo<usize>> for StaticVec<T, N> {
  type Output = [T];
  /// Asserts that the upper bound of `index` is less than or equal to the
  /// current length of the StaticVec, and if so returns a constant reference
  /// to a slice of elements `0..index.end`.
  #[inline(always)]
  fn index(&self, index: RangeTo<usize>) -> &Self::Output {
    assert!(index.end <= self.length);
    slice_from_raw_parts(self.as_ptr(), index.end)
  }
}

impl<T, const N: usize> IndexMut<RangeTo<usize>> for StaticVec<T, N> {
  /// Asserts that the upper bound of `index` is less than or equal to the
  /// current length of the StaticVec, and if so returns a constant reference
  /// to a slice of elements `0..index.end`.
  #[inline(always)]
  fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
    assert!(index.end <= self.length);
    slice_from_raw_parts_mut(self.as_mut_ptr(), index.end)
  }
}

impl<T, const N: usize> Index<RangeToInclusive<usize>> for StaticVec<T, N> {
  type Output = [T];
  /// Asserts that the upper bound of `index` is less than the
  /// current length of the StaticVec, and if so returns a constant reference
  /// to a slice of elements `0..=index.end`.
  #[inline(always)]
  fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
    assert!(index.end < self.length);
    slice_from_raw_parts(self.as_ptr(), index.end + 1)
  }
}

impl<T, const N: usize> IndexMut<RangeToInclusive<usize>> for StaticVec<T, N> {
  /// Asserts that the upper bound of `index` is less than the
  /// current length of the StaticVec, and if so returns a constant reference
  /// to a slice of elements `0..=index.end`.
  #[inline(always)]
  fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut Self::Output {
    assert!(index.end < self.length);
    slice_from_raw_parts_mut(self.as_mut_ptr(), index.end + 1)
  }
}

#[cfg(feature = "std")]
#[doc(cfg(feature = "std"))]
impl<T, const N: usize> Into<Vec<T>> for StaticVec<T, N> {
  /// Functionally equivalent to [`into_vec`](crate::StaticVec::into_vec).
  #[inline(always)]
  fn into(self) -> Vec<T> {
    self.into_vec()
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a StaticVec<T, N> {
  type IntoIter = StaticVecIterConst<'a, T, N>;
  type Item = &'a T;
  /// Returns a [`StaticVecIterConst`](crate::iterators::StaticVecIterConst) over the StaticVec's
  /// inhabited area.
  #[inline(always)]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a mut StaticVec<T, N> {
  type IntoIter = StaticVecIterMut<'a, T, N>;
  type Item = &'a mut T;
  /// Returns a [`StaticVecIterMut`](crate::iterators::StaticVecIterMut) over the StaticVec's
  /// inhabited area.
  #[inline(always)]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<T, const N: usize> IntoIterator for StaticVec<T, N> {
  type IntoIter = StaticVecIntoIter<T, N>;
  type Item = T;
  /// Returns a by-value [`StaticVecIntoIter`](crate::iterators::StaticVecIntoIter) over the
  /// StaticVec's inhabited area, which consumes the StaticVec.
  #[inline(always)]
  fn into_iter(mut self) -> Self::IntoIter {
    let old_length = self.length;
    // This prevents the values from being dropped locally, since they're
    // being copied into the iterator.
    self.length = 0;
    StaticVecIntoIter {
      start: 0,
      end: old_length,
      data: {
        // Copy the inhabited part of self into the iterator.
        let mut data = Self::new_data_uninit();
        unsafe {
          self
            .as_ptr()
            .copy_to_nonoverlapping(Self::first_ptr_mut(&mut data), old_length)
        };
        data
      },
    }
  }
}

impl<T: Ord, const N: usize> Ord for StaticVec<T, N> {
  #[inline(always)]
  fn cmp(&self, other: &Self) -> Ordering {
    Ord::cmp(self.as_slice(), other.as_slice())
  }
}

impl_partial_eq_with_as_slice!(StaticVec<T1, N1>, StaticVec<T2, N2>);
impl_partial_eq_with_as_slice!(StaticVec<T1, N1>, &StaticVec<T2, N2>);
impl_partial_eq_with_as_slice!(StaticVec<T1, N1>, &mut StaticVec<T2, N2>);
impl_partial_eq_with_as_slice!(&StaticVec<T1, N1>, StaticVec<T2, N2>);
impl_partial_eq_with_as_slice!(&mut StaticVec<T1, N1>, StaticVec<T2, N2>);
impl_partial_eq_with_get_unchecked!([T1; N1], StaticVec<T2, N2>);
impl_partial_eq_with_get_unchecked!([T1; N1], &StaticVec<T2, N2>);
impl_partial_eq_with_get_unchecked!([T1; N1], &mut StaticVec<T2, N2>);
impl_partial_eq_with_get_unchecked!(&[T1; N1], StaticVec<T2, N2>);
impl_partial_eq_with_get_unchecked!(&mut [T1; N1], StaticVec<T2, N2>);
impl_partial_eq_with_equals_no_deref!([T1], StaticVec<T2, N>);
impl_partial_eq_with_equals_no_deref!([T1], &StaticVec<T2, N>);
impl_partial_eq_with_equals_no_deref!([T1], &mut StaticVec<T2, N>);
impl_partial_eq_with_equals_deref!(&[T1], StaticVec<T2, N>);
impl_partial_eq_with_equals_deref!(&mut [T1], StaticVec<T2, N>);
impl_partial_ord_with_as_slice!(StaticVec<T1, N1>, StaticVec<T2, N2>);
impl_partial_ord_with_as_slice!(StaticVec<T1, N1>, &StaticVec<T2, N2>);
impl_partial_ord_with_as_slice!(StaticVec<T1, N1>, &mut StaticVec<T2, N2>);
impl_partial_ord_with_as_slice!(&StaticVec<T1, N1>, StaticVec<T2, N2>);
impl_partial_ord_with_as_slice!(&mut StaticVec<T1, N1>, StaticVec<T2, N2>);
impl_partial_ord_with_get_unchecked!([T1; N1], StaticVec<T2, N2>);
impl_partial_ord_with_get_unchecked!([T1; N1], &StaticVec<T2, N2>);
impl_partial_ord_with_get_unchecked!([T1; N1], &mut StaticVec<T2, N2>);
impl_partial_ord_with_get_unchecked!(&[T1; N1], StaticVec<T2, N2>);
impl_partial_ord_with_get_unchecked!(&mut [T1; N1], StaticVec<T2, N2>);
impl_partial_ord_with_as_slice_against_slice!([T1], StaticVec<T2, N>);
impl_partial_ord_with_as_slice_against_slice!([T1], &StaticVec<T2, N>);
impl_partial_ord_with_as_slice_against_slice!([T1], &mut StaticVec<T2, N>);
impl_partial_ord_with_as_slice_against_slice!(&[T1], StaticVec<T2, N>);
impl_partial_ord_with_as_slice_against_slice!(&mut [T1], StaticVec<T2, N>);

/// Read from a StaticVec. This implementation operates by copying bytes into the destination
/// buffers, then shifting the remaining bytes over.
#[cfg(feature = "std")]
impl<const N: usize> Read for StaticVec<u8, N> {
  #[inline(always)]
  unsafe fn initializer(&self) -> io::Initializer {
    io::Initializer::nop()
  }

  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let current_length = self.length;
    let read_length = current_length.min(buf.len());
    // Safety:  read_length <= buf.length and self.length. Rust borrowing
    // rules mean that buf is guaranteed not to overlap with self.
    unsafe {
      buf
        .as_mut_ptr()
        .copy_from_nonoverlapping(self.as_ptr(), read_length);
    }
    if read_length < current_length {
      // Safety: we just confirmed that read_length is less than our current length.
      unsafe {
        self
          .ptr_at_unchecked(read_length)
          .copy_to(self.as_mut_ptr(), current_length - read_length)
      };
    }
    // Safety: 0 <= read_length <= current_length
    unsafe { self.set_len(current_length - read_length) };
    Ok(read_length)
  }

  #[inline]
  fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
    let read_length = self.length;
    buf.extend_from_slice(self.as_slice());
    self.length = 0;
    Ok(read_length)
  }

  #[inline]
  fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
    let read_length = self.length;
    match str::from_utf8(self.as_slice()) {
      Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
      Ok(self_str) => buf.push_str(self_str),
    };
    self.length = 0;
    Ok(read_length)
  }

  #[inline]
  fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
    if buf.len() > self.length {
      Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "Not enough data available to fill the provided buffer!",
      ))
    } else {
      // read is guaranteed to fully read into the buf in a single call
      self.read(buf).and(Ok(()))
    }
  }

  #[inline]
  fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
    // Minimize copies: copy to each output buf in sequence, then shift the
    // internal data only once. This as opposed to calling `read` in a loop,
    // which shifts the inner data each time.
    let mut start_ptr = self.as_ptr();
    let old_length = self.length;
    // We update self.length inplace in the loop to track how many bytes
    // have been written. This means that when we perform the shift at the
    // end, self.length is already correct.
    for buf in bufs {
      if self.is_empty() {
        break;
      }
      // The number of bytes we'll be reading out of self.
      let read_length = self.length.min(buf.len());
      // Safety: start_ptr is known to point to the array in self, which
      // is different than `buf`. read_length <= self.length.
      unsafe {
        buf
          .as_mut_ptr()
          .copy_from_nonoverlapping(start_ptr, read_length);
        start_ptr = start_ptr.add(read_length);
        self.length -= read_length;
      }
    }
    let current_length = self.length;
    let total_read = old_length - current_length;
    if current_length > 0 {
      unsafe {
        self
          .ptr_at_unchecked(total_read)
          .copy_to(self.as_mut_ptr(), current_length)
      };
    }
    Ok(total_read)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> Write for StaticVec<u8, N> {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let old_length = self.length;
    self.extend_from_slice(buf);
    Ok(self.length - old_length)
  }

  #[inline]
  fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    let old_length = self.length;
    for buf in bufs {
      if self.is_full() {
        break;
      }
      self.extend_from_slice(buf);
    }
    Ok(self.length - old_length)
  }

  #[inline]
  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    if buf.len() <= self.remaining_capacity() {
      self.extend_from_slice(buf);
      Ok(())
    } else {
      Err(io::Error::new(
        io::ErrorKind::WriteZero,
        "Insufficient remaining capacity!",
      ))
    }
  }

  #[inline(always)]
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "std")]
impl<const N: usize> BufRead for StaticVec<u8, N> {
  #[inline(always)]
  fn fill_buf(&mut self) -> io::Result<&[u8]> {
    Ok(&**self)
  }

  #[inline(always)]
  fn consume(&mut self, amt: usize) {
    *self = Self::new_from_slice(&self[amt..]);
  }
}

#[cfg(feature = "serde_support")]
impl<'de, T, const N: usize> Deserialize<'de> for StaticVec<T, N>
where T: Deserialize<'de>
{
  #[inline]
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: Deserializer<'de> {
    struct StaticVecVisitor<'de, T, const N: usize>(PhantomData<(&'de (), T)>);

    impl<'de, T, const N: usize> Visitor<'de> for StaticVecVisitor<'de, T, N>
    where T: Deserialize<'de>
    {
      type Value = StaticVec<T, N>;

      #[inline(always)]
      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "no more than {} items", N)
      }

      #[inline]
      fn visit_seq<SA>(self, mut seq: SA) -> Result<Self::Value, SA::Error>
      where SA: SeqAccess<'de> {
        let mut res = Self::Value::new();
        while res.length < N {
          if let Some(val) = seq.next_element()? {
            unsafe { res.push_unchecked(val) };
          } else {
            break;
          }
        }
        Ok(res)
      }
    }
    deserializer.deserialize_seq(StaticVecVisitor::<T, N>(PhantomData))
  }
}

#[cfg(feature = "serde_support")]
impl<T, const N: usize> Serialize for StaticVec<T, N>
where T: Serialize
{
  #[inline(always)]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer {
    serializer.collect_seq(self)
  }
}
