use crate::iterators::*;
use crate::StaticVec;
use core::cmp::{Eq, Ord, Ordering, PartialEq};
use core::fmt::{Debug, Formatter, Result};
use core::iter::FromIterator;
use core::ops::{Index, IndexMut};

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

impl<T, const N: usize> Extend<T> for StaticVec<T, {N}> {
  ///Appends all elements, if any, from `iter` to the StaticVec. If `iter` has a size greater than
  ///the StaticVec's capacity, any items after that point are ignored.
  #[inline]
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    let mut it = iter.into_iter();
    let iter_length = it.size_hint().0;
    if iter_length > 0 {
      let new_length = (self.length + iter_length).min(N);
      for i in self.length..new_length {
        unsafe {
          self.data.get_unchecked_mut(i).write(it.next().unwrap());
        }
      }
      self.length = new_length;
    } else {
      for i in self.length..N {
        if let Some(val) = it.next() {
          unsafe {
            self.data.get_unchecked_mut(i).write(val);
          }
        } else {
          self.length = i;
        }
      }
    }
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

impl<T, const N: usize> FromIterator<T> for StaticVec<T, {N}> {
  ///Creates a new StaticVec instance from the elements, if any, of `iter`.
  ///If `iter` has a size greater than the StaticVec's capacity, any items after
  ///that point are ignored.
  #[inline]
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut res = Self::new();
    let mut it = iter.into_iter();
    for i in 0..N {
      if let Some(val) = it.next() {
        unsafe {
          res.data.get_unchecked_mut(i).write(val);
        }
      } else {
        res.length = i;
        return res;
      }
    }
    res.length = N;
    res
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

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<StaticVec<T, {N1}>>
  for StaticVec<T, {N2}>
{
  #[inline(always)]
  fn eq(&self, other: &StaticVec<T, {N1}>) -> bool {
    self.as_slice() == other.as_slice()
  }
  #[inline(always)]
  fn ne(&self, other: &StaticVec<T, {N1}>) -> bool {
    self.as_slice() != other.as_slice()
  }
}

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<&StaticVec<T, {N1}>>
  for StaticVec<T, {N2}>
{
  #[inline(always)]
  fn eq(&self, other: &&StaticVec<T, {N1}>) -> bool {
    self.as_slice() == other.as_slice()
  }
  #[inline(always)]
  fn ne(&self, other: &&StaticVec<T, {N1}>) -> bool {
    self.as_slice() != other.as_slice()
  }
}

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<&mut StaticVec<T, {N1}>>
  for StaticVec<T, {N2}>
{
  #[inline(always)]
  fn eq(&self, other: &&mut StaticVec<T, {N1}>) -> bool {
    self.as_slice() == other.as_slice()
  }
  #[inline(always)]
  fn ne(&self, other: &&mut StaticVec<T, {N1}>) -> bool {
    self.as_slice() != other.as_slice()
  }
}

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<[T; N1]> for StaticVec<T, {N2}> {
  #[inline(always)]
  fn eq(&self, other: &[T; N1]) -> bool {
    unsafe { self.as_slice() == other.get_unchecked(..) }
  }
  #[inline(always)]
  fn ne(&self, other: &[T; N1]) -> bool {
    unsafe { self.as_slice() != other.get_unchecked(..) }
  }
}

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<&[T; N1]> for StaticVec<T, {N2}> {
  #[inline(always)]
  fn eq(&self, other: &&[T; N1]) -> bool {
    unsafe { self.as_slice() == other.get_unchecked(..) }
  }
  #[inline(always)]
  fn ne(&self, other: &&[T; N1]) -> bool {
    unsafe { self.as_slice() != other.get_unchecked(..) }
  }
}

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<&mut [T; N1]> for StaticVec<T, {N2}> {
  #[inline(always)]
  fn eq(&self, other: &&mut [T; N1]) -> bool {
    unsafe { self.as_slice() == other.get_unchecked(..) }
  }
  #[inline(always)]
  fn ne(&self, other: &&mut [T; N1]) -> bool {
    unsafe { self.as_slice() != other.get_unchecked(..) }
  }
}

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<[T; N1]> for &StaticVec<T, {N2}> {
  #[inline(always)]
  fn eq(&self, other: &[T; N1]) -> bool {
    unsafe { self.as_slice() == other.get_unchecked(..) }
  }
  #[inline(always)]
  fn ne(&self, other: &[T; N1]) -> bool {
    unsafe { self.as_slice() != other.get_unchecked(..) }
  }
}

impl<T: PartialEq, const N1: usize, const N2: usize> PartialEq<[T; N1]> for &mut StaticVec<T, {N2}> {
  #[inline(always)]
  fn eq(&self, other: &[T; N1]) -> bool {
    unsafe { self.as_slice() == other.get_unchecked(..) }
  }
  #[inline(always)]
  fn ne(&self, other: &[T; N1]) -> bool {
    unsafe { self.as_slice() != other.get_unchecked(..) }
  }
}

impl<T: PartialEq, const N: usize> PartialEq<[T]> for StaticVec<T, {N}> {
  #[inline(always)]
  fn eq(&self, other: &[T]) -> bool {
    self.as_slice() == other
  }
  #[inline(always)]
  fn ne(&self, other: &[T]) -> bool {
    self.as_slice() != other
  }
}

impl<T: PartialEq, const N: usize> PartialEq<&[T]> for StaticVec<T, {N}> {
  #[inline(always)]
  fn eq(&self, other: &&[T]) -> bool {
    self.as_slice() == *other
  }
  #[inline(always)]
  fn ne(&self, other: &&[T]) -> bool {
    self.as_slice() != *other
  }
}

impl<T: PartialEq, const N: usize> PartialEq<&mut [T]> for StaticVec<T, {N}> {
  #[inline(always)]
  fn eq(&self, other: &&mut [T]) -> bool {
    self.as_slice() == *other
  }
  #[inline(always)]
  fn ne(&self, other: &&mut [T]) -> bool {
    self.as_slice() != *other
  }
}

impl<T: PartialEq, const N: usize> PartialEq<[T]> for &StaticVec<T, {N}> {
  #[inline(always)]
  fn eq(&self, other: &[T]) -> bool {
    self.as_slice() == other
  }
  #[inline(always)]
  fn ne(&self, other: &[T]) -> bool {
    self.as_slice() != other
  }
}

impl<T: PartialEq, const N: usize> PartialEq<[T]> for &mut StaticVec<T, {N}> {
  #[inline(always)]
  fn eq(&self, other: &[T]) -> bool {
    self.as_slice() == other
  }
  #[inline(always)]
  fn ne(&self, other: &[T]) -> bool {
    self.as_slice() != other
  }
}

impl<T: Ord, const N: usize> Ord for StaticVec<T, {N}> {
  #[inline(always)]
  fn cmp(&self, other: &StaticVec<T, {N}>) -> Ordering {
    Ord::cmp(self.as_slice(), other.as_slice())
  }
}

impl<T: PartialOrd, const N1: usize, const N2: usize> PartialOrd<StaticVec<T, {N1}>>
  for StaticVec<T, {N2}>
{
  #[inline(always)]
  fn partial_cmp(&self, other: &StaticVec<T, {N1}>) -> Option<Ordering> {
    PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
  }
}

impl<T: PartialOrd, const N1: usize, const N2: usize> PartialOrd<&StaticVec<T, {N1}>>
  for StaticVec<T, {N2}>
{
  #[inline(always)]
  fn partial_cmp(&self, other: &&StaticVec<T, {N1}>) -> Option<Ordering> {
    PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
  }
}

impl<T: PartialOrd, const N1: usize, const N2: usize> PartialOrd<&mut StaticVec<T, {N1}>>
  for StaticVec<T, {N2}>
{
  #[inline(always)]
  fn partial_cmp(&self, other: &&mut StaticVec<T, {N1}>) -> Option<Ordering> {
    PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
  }
}
