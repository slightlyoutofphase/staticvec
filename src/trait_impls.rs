use crate::utils::partial_compare;
use crate::StaticVec;
use core::cmp::{Eq, Ord, Ordering, PartialEq};
use core::fmt::{self, Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice;

#[cfg(feature = "std")]
use std::io::{self, IoSlice, Write};

#[cfg(feature = "serde_support")]
use core::marker::PhantomData;

#[cfg(feature = "serde_support")]
use serde::{
  de::{SeqAccess, Visitor},
  Deserialize, Deserializer, Serialize, Serializer,
};

impl<T, const N: usize> AsMut<[T]> for StaticVec<T, { N }> {
  #[inline(always)]
  fn as_mut(&mut self) -> &mut [T] {
    self.as_mut_slice()
  }
}

impl<T, const N: usize> AsRef<[T]> for StaticVec<T, { N }> {
  #[inline(always)]
  fn as_ref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<T, const N: usize> Deref for StaticVec<T, { N }> {
  type Target = [T];

  #[inline(always)]
  fn deref(&self) -> &[T] {
    self.as_slice()
  }
}

impl<T, const N: usize> DerefMut for StaticVec<T, { N }> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut [T] {
    self.as_mut()
  }
}

impl<T: Clone, const N: usize> Clone for StaticVec<T, { N }> {
  #[inline]
  fn clone(&self) -> Self {
    let mut res = Self::new();

    for item in self {
      // Safety: self is the same type as res, so it can never go over capacity
      unsafe { res.push_unchecked(item.clone()) };
    }

    res
  }

  #[inline]
  fn clone_from(&mut self, rhs: &Self) {
    self.truncate(rhs.length);

    for i in 0..self.length {
      // Safety: after the truncate, self.len <= rhs.len, which means that
      // for every i in self, there is definitely an element at rhs[i]
      unsafe {
        self.get_unchecked_mut(i).clone_from(rhs.get_unchecked(i));
      }
    }

    for i in self.length..rhs.length {
      // Safety: i < rhs.length, so get_unchecked is safe. i starts at
      // self.length, which is <= rhs.length, so there is always an available
      // slot at self[i]
      unsafe { self.push_unchecked(rhs.get_unchecked(i).clone()) }
    }
  }
}

impl<T: Debug, const N: usize> Debug for StaticVec<T, { N }> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.as_slice().fmt(f)
  }
}

impl<T, const N: usize> Default for StaticVec<T, { N }> {
  ///Calls `new`.
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

impl<T, const N: usize> Drop for StaticVec<T, { N }> {
  ///Calls `clear` through the StaticVec before dropping it.
  #[inline(always)]
  fn drop(&mut self) {
    // This sets length to 0 unnecessarily, but we asume the optimizer
    // will take care of it.
    self.clear();
  }
}

impl<T: Eq, const N: usize> Eq for StaticVec<T, { N }> {}

//TODO: Figure out how to handle "may or may not need explicit dereferencing" in macros,
//so that I can macro-ize the two Extend implementations below.

impl<T, const N: usize> Extend<T> for StaticVec<T, { N }> {
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

impl<'a, T: 'a + Copy, const N: usize> Extend<&'a T> for StaticVec<T, { N }> {
  ///Appends all elements, if any, from `iter` to the StaticVec. If `iter` has a size greater than
  ///the StaticVec's capacity, any items after that point are ignored.
  #[inline]
  fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
    self.extend(iter.into_iter().copied());
  }
}

impl<T: Copy, const N: usize> From<&[T]> for StaticVec<T, { N }> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_slice](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &[T]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T: Copy, const N: usize> From<&mut [T]> for StaticVec<T, { N }> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_mut_slice](crate::StaticVec::new_from_mut_slice) internally.
  #[inline(always)]
  fn from(values: &mut [T]) -> Self {
    Self::new_from_mut_slice(values)
  }
}

impl<T, const N1: usize, const N2: usize> From<[T; N1]> for StaticVec<T, { N2 }> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_array](crate::StaticVec::new_from_array) internally.
  #[inline(always)]
  fn from(values: [T; N1]) -> Self {
    Self::new_from_array(values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<&[T; N1]> for StaticVec<T, { N2 }> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_array](crate::StaticVec::new_from_array) internally.
  #[inline(always)]
  fn from(values: &[T; N1]) -> Self {
    Self::new_from_array(*values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<&mut [T; N1]> for StaticVec<T, { N2 }> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_array](crate::StaticVec::new_from_array) internally.
  #[inline(always)]
  fn from(values: &mut [T; N1]) -> Self {
    Self::new_from_array(*values)
  }
}

impl<T, U, const N: usize> FromIterator<U> for StaticVec<T, { N }>
where Self: Extend<U>
{
  ///Creates a new StaticVec instance from the elements, if any, of `iter`.
  ///If `iter` has a size greater than the StaticVec's capacity, any items after
  ///that point are ignored.
  #[inline]
  fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
    let mut res = Self::new();
    res.extend(iter);
    res
  }
}

impl<T: Hash, const N: usize> Hash for StaticVec<T, { N }> {
  #[inline(always)]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state);
  }
}

impl<T, I: slice::SliceIndex<[T]>, const N: usize> Index<I> for StaticVec<T, { N }> {
  type Output = I::Output;

  #[inline(always)]
  fn index(&self, index: I) -> &I::Output {
    &self.as_slice()[index]
  }
}

impl<T, I: slice::SliceIndex<[T]>, const N: usize> IndexMut<I> for StaticVec<T, { N }> {
  #[inline(always)]
  fn index_mut(&mut self, index: I) -> &mut I::Output {
    &mut self.as_mut_slice()[index]
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a StaticVec<T, { N }> {
  type IntoIter = slice::Iter<'a, T>;
  type Item = &'a T;
  ///Returns a `StaticVecIterConst` over the StaticVec's inhabited area.
  #[inline(always)]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a mut StaticVec<T, { N }> {
  type IntoIter = slice::IterMut<'a, T>;
  type Item = &'a mut T;
  ///Returns a `StaticVecIterMut` over the StaticVec's inhabited area.
  #[inline(always)]
  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl<T: Ord, const N: usize> Ord for StaticVec<T, { N }> {
  #[inline(always)]
  fn cmp(&self, other: &Self) -> Ordering {
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
      #[allow(clippy::partialeq_ne_impl)]
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
      #[allow(clippy::partialeq_ne_impl)]
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
      #[allow(clippy::partialeq_ne_impl)]
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
      #[allow(clippy::partialeq_ne_impl)]
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
impl<const N: usize> Write for StaticVec<u8, { N }> {
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
      if self.is_full() {
        break;
      }
    }
    Ok(self.length - old_length)
  }

  #[inline(always)]
  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    // We need at-most one write to `self`, and this `write` is infallible.
    if self.write(buf)? == buf.len() {
      Ok(())
    } else {
      Err(io::Error::new(
        io::ErrorKind::WriteZero,
        "Not enough capacity left in StaticVec",
      ))
    }
  }

  #[inline(always)]
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "serde_support")]
impl<'de, T, const N: usize> Deserialize<'de> for StaticVec<T, { N }>
where T: Deserialize<'de>
{
  #[inline]
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: Deserializer<'de> {
    struct StaticVecVisitor<'de, T, const N: usize>(PhantomData<(&'de (), T)>);

    impl<'de, T, const N: usize> Visitor<'de> for StaticVecVisitor<'de, T, { N }>
    where T: Deserialize<'de>
    {
      type Value = StaticVec<T, { N }>;

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
            unsafe {
              res.push_unchecked(val);
            }
          } else {
            break;
          }
        }
        Ok(res)
      }
    }
    deserializer.deserialize_seq(StaticVecVisitor::<T, { N }>(PhantomData))
  }
}

#[cfg(feature = "serde_support")]
impl<T, const N: usize> Serialize for StaticVec<T, { N }>
where T: Serialize
{
  #[inline(always)]
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer {
    serializer.collect_seq(self)
  }
}
