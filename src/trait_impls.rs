use crate::iterators::*;
use crate::utils::partial_compare;
use crate::StaticVec;
use core::cmp::{Eq, Ord, Ordering, PartialEq};
use core::fmt::{Debug, Formatter, Result};
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut, Range, RangeFull, RangeInclusive};
#[cfg(feature = "std")]
use std::io::{self, IoSlice, Read, Write};

impl<T, const N: usize> AsMut<T> for StaticVec<T, {N}> {
  ///Asserts that the StaticVec's current length is greater than 0 to avoid
  ///returning an invalid reference, and if so calls [as_mut_ptr](crate::StaticVec::as_mut_ptr)
  ///internally and converts it to one.
  #[inline(always)]
  fn as_mut(&mut self) -> &mut T {
    //I don't know how useful this impl actually is. We'll see I guess.
    assert!(self.length > 0);
    unsafe { self.as_mut_ptr().as_mut().unwrap() }
  }
}

impl<T, const N: usize> AsRef<T> for StaticVec<T, {N}> {
  ///Asserts that the StaticVec's current length is greater than 0 to avoid
  ///returning an invalid reference, and if so calls [as_ptr](crate::StaticVec::as_ptr)
  ///internally and converts it to one.
  #[inline(always)]
  fn as_ref(&self) -> &T {
    //I don't know how useful this impl actually is. We'll see I guess.
    assert!(self.length > 0);
    unsafe { self.as_ptr().as_ref().unwrap() }
  }
}

impl<T, const N: usize> AsMut<[T]> for StaticVec<T, {N}> {
  #[inline(always)]
  fn as_mut(&mut self) -> &mut [T] {
    self.as_mut_slice()
  }
}

impl<T, const N: usize> AsRef<[T]> for StaticVec<T, {N}> {
  #[inline(always)]
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<T: Clone, const N: usize> Clone for StaticVec<T, {N}> {
  #[inline]
  fn clone(&self) -> Self {
    let mut res = Self::new();
    for i in 0..self.length {
      unsafe {
        res
          .data
          .get_unchecked_mut(i)
          .write(self.data.get_unchecked(i).get_ref().clone());
        res.length += 1;
      }
    }
    res
  }
}

impl<T: Debug, const N: usize> Debug for StaticVec<T, {N}> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> Result {
    Debug::fmt(self.as_slice(), f)
  }
}

impl<T, const N: usize> Default for StaticVec<T, {N}> {
  ///Calls `new`.
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const N: usize> Drop for StaticVec<T, {N}> {
  ///Calls `clear` through the StaticVec before dropping it.
  #[inline(always)]
  fn drop(&mut self) {
    self.clear();
  }
}

impl<T: Eq, const N: usize> Eq for StaticVec<T, {N}> {}

//TODO: Figure out how to handle "may or may not need explicit dereferencing" in macros,
//so that I can macro-ize the two Extend implementations below.

impl<T, const N: usize> Extend<T> for StaticVec<T, {N}> {
  ///Appends all elements, if any, from `iter` to the StaticVec. If `iter` has a size greater than
  ///the StaticVec's capacity, any items after that point are ignored.
  #[inline]
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    let mut it = iter.into_iter();
    let mut i = self.length;
    while i < N {
      if let Some(val) = it.next() {
        unsafe {
          self.data.get_unchecked_mut(i).write(val);
        }
      } else {
        break;
      }
      i += 1;
    }
    self.length = i;
  }
}

impl<'a, T: 'a + Copy, const N: usize> Extend<&'a T> for StaticVec<T, {N}> {
  ///Appends all elements, if any, from `iter` to the StaticVec. If `iter` has a size greater than
  ///the StaticVec's capacity, any items after that point are ignored.
  #[inline]
  fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
    let mut it = iter.into_iter();
    let mut i = self.length;
    while i < N {
      if let Some(val) = it.next() {
        unsafe {
          self.data.get_unchecked_mut(i).write(*val);
        }
      } else {
        break;
      }
      i += 1;
    }
    self.length = i;
  }
}

impl<T: Copy, const N: usize> From<&[T]> for StaticVec<T, {N}> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_slice](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &[T]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T: Copy, const N: usize> From<&mut [T]> for StaticVec<T, {N}> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_mut_slice](crate::StaticVec::new_from_mut_slice) internally.
  #[inline(always)]
  fn from(values: &mut [T]) -> Self {
    Self::new_from_mut_slice(values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<[T; N1]> for StaticVec<T, {N2}> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_array](crate::StaticVec::new_from_array) internally.
  #[inline(always)]
  fn from(values: [T; N1]) -> Self {
    Self::new_from_array(values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<&[T; N1]> for StaticVec<T, {N2}> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_array](crate::StaticVec::new_from_array) internally.
  #[inline(always)]
  fn from(values: &[T; N1]) -> Self {
    Self::new_from_array(*values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<&mut [T; N1]> for StaticVec<T, {N2}> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_array](crate::StaticVec::new_from_array) internally.
  #[inline(always)]
  fn from(values: &mut [T; N1]) -> Self {
    Self::new_from_array(*values)
  }
}

//TODO: Figure out how to handle "may or may not need explicit dereferencing" in macros,
//so that I can macro-ize the two FromIterator implementations below.

impl<T, const N: usize> FromIterator<T> for StaticVec<T, {N}> {
  ///Creates a new StaticVec instance from the elements, if any, of `iter`.
  ///If `iter` has a size greater than the StaticVec's capacity, any items after
  ///that point are ignored.
  #[inline]
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut res = Self::new();
    let mut it = iter.into_iter();
    let mut i = 0;
    while i < N {
      if let Some(val) = it.next() {
        unsafe {
          res.data.get_unchecked_mut(i).write(val);
        }
      } else {
        break;
      }
      i += 1;
    }
    res.length = i;
    res
  }
}

impl<'a, T: 'a + Copy, const N: usize> FromIterator<&'a T> for StaticVec<T, {N}> {
  ///Creates a new StaticVec instance from the elements, if any, of `iter`.
  ///If `iter` has a size greater than the StaticVec's capacity, any items after
  ///that point are ignored.
  #[inline]
  fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
    let mut res = Self::new();
    let mut it = iter.into_iter();
    let mut i = 0;
    while i < N {
      if let Some(val) = it.next() {
        unsafe {
          res.data.get_unchecked_mut(i).write(*val);
        }
      } else {
        break;
      }
      i += 1;
    }
    res.length = i;
    res
  }
}

impl<T: Hash, const N: usize> Hash for StaticVec<T, {N}> {
  #[inline(always)]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state);
  }
}

impl<T, const N: usize> Index<usize> for StaticVec<T, {N}> {
  type Output = T;
  ///Asserts that `index` is less than the current length of the StaticVec,
  ///and if so returns the value at that position as a constant reference.
  #[inline(always)]
  fn index(&self, index: usize) -> &Self::Output {
    assert!(index < self.length);
    unsafe { self.data.get_unchecked(index).get_ref() }
  }
}

impl<T, const N: usize> IndexMut<usize> for StaticVec<T, {N}> {
  ///Asserts that `index` is less than the current length of the StaticVec,
  ///and if so returns the value at that position as a mutable reference.
  #[inline(always)]
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    assert!(index < self.length);
    unsafe { self.data.get_unchecked_mut(index).get_mut() }
  }
}

impl<T, const N: usize> Index<Range<usize>> for StaticVec<T, {N}> {
  type Output = [T];
  ///Asserts that the lower bound of `index` is less than its upper bound,
  ///and that its upper bound is less than or equal to the current length of the StaticVec,
  ///and if so returns a constant reference to a slice of elements `index.start..index.end`.
  #[inline(always)]
  fn index(&self, index: Range<usize>) -> &Self::Output {
    assert!(index.start < index.end && index.end <= self.length);
    unsafe { &*(self.data.get_unchecked(index) as *const [MaybeUninit<T>] as *const [T]) }
  }
}

impl<T, const N: usize> IndexMut<Range<usize>> for StaticVec<T, {N}> {
  ///Asserts that the lower bound of `index` is less than its upper bound,
  ///and that its upper bound is less than or equal to the current length of the StaticVec,
  ///and if so returns a mutable reference to a slice of elements `index.start..index.end`.
  #[inline(always)]
  fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
    assert!(index.start < index.end && index.end <= self.length);
    unsafe { &mut *(self.data.get_unchecked_mut(index) as *mut [MaybeUninit<T>] as *mut [T]) }
  }
}

impl<T, const N: usize> Index<RangeFull> for StaticVec<T, {N}> {
  type Output = [T];
  ///Returns a constant reference to a slice consisting of `0..self.length`
  //elements of the StaticVec, using [as_slice](crate::StaticVec::as_slice) internally.
  #[inline(always)]
  fn index(&self, _index: RangeFull) -> &Self::Output {
    self.as_slice()
  }
}

impl<T, const N: usize> IndexMut<RangeFull> for StaticVec<T, {N}> {
  ///Returns a mutable reference to a slice consisting of `0..self.length`
  //elements of the StaticVec, using [as_mut_slice](crate::StaticVec::as_mut_slice) internally.
  #[inline(always)]
  fn index_mut(&mut self, _index: RangeFull) -> &mut Self::Output {
    self.as_mut_slice()
  }
}

impl<T, const N: usize> Index<RangeInclusive<usize>> for StaticVec<T, {N}> {
  type Output = [T];
  ///Asserts that the lower bound of `index` is less than or equal to its upper bound,
  //and that its upper bound is less than the current length of the StaticVec,
  ///and if so returns a constant reference to a slice of elements `index.start()..=index.end()`.
  #[inline(always)]
  fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
    assert!(index.start() <= index.end() && index.end() < &self.length);
    unsafe { &*(self.data.get_unchecked(index) as *const [MaybeUninit<T>] as *const [T]) }
  }
}

impl<T, const N: usize> IndexMut<RangeInclusive<usize>> for StaticVec<T, {N}> {
  ///Asserts that the lower bound of `index` is less than or equal to its upper bound,
  //and that its upper bound is less than the current length of the StaticVec,
  ///and if so returns a mutable reference to a slice of elements `index.start()..=index.end()`.
  #[inline(always)]
  fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
    assert!(index.start() <= index.end() && index.end() < &self.length);
    unsafe { &mut *(self.data.get_unchecked_mut(index) as *mut [MaybeUninit<T>] as *mut [T]) }
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a StaticVec<T, {N}> {
  type IntoIter = StaticVecIterConst<'a, T>;
  type Item = <Self::IntoIter as Iterator>::Item;
  ///Returns a `StaticVecIterConst` over the StaticVec's inhabited area.
  #[inline(always)]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a mut StaticVec<T, {N}> {
  type IntoIter = StaticVecIterMut<'a, T>;
  type Item = <Self::IntoIter as Iterator>::Item;
  ///Returns a `StaticVecIterMut` over the StaticVec's inhabited area.
  #[inline(always)]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<T: Ord, const N: usize> Ord for StaticVec<T, {N}> {
  #[inline(always)]
  fn cmp(&self, other: &StaticVec<T, {N}>) -> Ordering {
    Ord::cmp(self.as_slice(), other.as_slice())
  }
}

macro_rules! impl_partial_eq_with_as_slice {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N1: usize, const N2: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        self.as_slice() == other.as_slice()
      }
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        self.as_slice() != other.as_slice()
      }
    }
  };
}

macro_rules! impl_partial_eq_with_get_unchecked {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N1: usize, const N2: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        unsafe { self.as_slice() == other.get_unchecked(..) }
      }
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        unsafe { self.as_slice() != other.get_unchecked(..) }
      }
    }
  };
}

macro_rules! impl_partial_eq_with_equals_no_deref {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        self.as_slice() == other
      }
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        self.as_slice() != other
      }
    }
  };
}

macro_rules! impl_partial_eq_with_equals_deref {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        self.as_slice() == *other
      }
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        self.as_slice() != *other
      }
    }
  };
}

macro_rules! impl_partial_ord_with_as_slice {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialOrd<T1>, const N1: usize, const N2: usize> PartialOrd<$left> for $right {
      #[inline(always)]
      fn partial_cmp(&self, other: &$left) -> Option<Ordering> {
        partial_compare(self.as_slice(), other.as_slice())
      }
    }
  };
}

macro_rules! impl_partial_ord_with_get_unchecked {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialOrd<T1>, const N1: usize, const N2: usize> PartialOrd<$left> for $right {
      #[inline(always)]
      fn partial_cmp(&self, other: &$left) -> Option<Ordering> {
        unsafe { partial_compare(self.as_slice(), other.get_unchecked(..)) }
      }
    }
  };
}

macro_rules! impl_partial_ord_with_as_slice_against_slice {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialOrd<T1>, const N: usize> PartialOrd<$left> for $right {
      #[inline(always)]
      fn partial_cmp(&self, other: &$left) -> Option<Ordering> {
        partial_compare(self.as_slice(), other)
      }
    }
  };
}

impl_partial_eq_with_as_slice!(StaticVec<T1, {N1}>, StaticVec<T2, {N2}>);
impl_partial_eq_with_as_slice!(StaticVec<T1, {N1}>, &StaticVec<T2, {N2}>);
impl_partial_eq_with_as_slice!(StaticVec<T1, {N1}>, &mut StaticVec<T2, {N2}>);
impl_partial_eq_with_as_slice!(&StaticVec<T1, {N1}>, StaticVec<T2, {N2}>);
impl_partial_eq_with_as_slice!(&mut StaticVec<T1, {N1}>, StaticVec<T2, {N2}>);
impl_partial_eq_with_get_unchecked!([T1; N1], StaticVec<T2, {N2}>);
impl_partial_eq_with_get_unchecked!([T1; N1], &StaticVec<T2, {N2}>);
impl_partial_eq_with_get_unchecked!([T1; N1], &mut StaticVec<T2, {N2}>);
impl_partial_eq_with_get_unchecked!(&[T1; N1], StaticVec<T2, {N2}>);
impl_partial_eq_with_get_unchecked!(&mut [T1; N1], StaticVec<T2, {N2}>);
impl_partial_eq_with_equals_no_deref!([T1], StaticVec<T2, {N}>);
impl_partial_eq_with_equals_no_deref!([T1], &StaticVec<T2, {N}>);
impl_partial_eq_with_equals_no_deref!([T1], &mut StaticVec<T2, {N}>);
impl_partial_eq_with_equals_deref!(&[T1], StaticVec<T2, {N}>);
impl_partial_eq_with_equals_deref!(&mut [T1], StaticVec<T2, {N}>);
impl_partial_ord_with_as_slice!(StaticVec<T1, {N1}>, StaticVec<T2, {N2}>);
impl_partial_ord_with_as_slice!(StaticVec<T1, {N1}>, &StaticVec<T2, {N2}>);
impl_partial_ord_with_as_slice!(StaticVec<T1, {N1}>, &mut StaticVec<T2, {N2}>);
impl_partial_ord_with_as_slice!(&StaticVec<T1, {N1}>, StaticVec<T2, {N2}>);
impl_partial_ord_with_as_slice!(&mut StaticVec<T1, {N1}>, StaticVec<T2, {N2}>);
impl_partial_ord_with_get_unchecked!([T1; N1], StaticVec<T2, {N2}>);
impl_partial_ord_with_get_unchecked!([T1; N1], &StaticVec<T2, {N2}>);
impl_partial_ord_with_get_unchecked!([T1; N1], &mut StaticVec<T2, {N2}>);
impl_partial_ord_with_get_unchecked!(&[T1; N1], StaticVec<T2, {N2}>);
impl_partial_ord_with_get_unchecked!(&mut [T1; N1], StaticVec<T2, {N2}>);
impl_partial_ord_with_as_slice_against_slice!([T1], StaticVec<T2, {N}>);
impl_partial_ord_with_as_slice_against_slice!([T1], &StaticVec<T2, {N}>);
impl_partial_ord_with_as_slice_against_slice!([T1], &mut StaticVec<T2, {N}>);
impl_partial_ord_with_as_slice_against_slice!(&[T1], StaticVec<T2, {N}>);
impl_partial_ord_with_as_slice_against_slice!(&mut [T1], StaticVec<T2, {N}>);

#[cfg(feature = "std")]
impl<const N: usize> Read for StaticVec<u8, {N}> {
  #[inline(always)]
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let read_length = self.length.min(buf.len());
    unsafe {
      self
        .as_ptr()
        .copy_to_nonoverlapping(buf.as_mut_ptr(), read_length);
    }
    Ok(read_length)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> Write for StaticVec<u8, {N}> {
  #[inline(always)]
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let old_length = self.length;
    self.extend_from_slice(buf);
    Ok(self.length - old_length)
  }

  #[inline]
  fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    let old_length = self.length;
    for buf in bufs {
      self.extend_from_slice(buf);
    }
    Ok(self.length - old_length)
  }

  #[inline(always)]
  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    self.extend_from_slice(buf);
    Ok(())
  }

  #[inline(always)]
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
