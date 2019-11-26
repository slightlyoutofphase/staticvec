use crate::iterators::*;
use crate::utils::partial_compare;
use crate::StaticVec;
use core::cmp::{Eq, Ord, Ordering, PartialEq};
use core::fmt::{self, Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::ptr;
use core::slice::SliceIndex;
use core::str;

#[cfg(feature = "std")]
use alloc::string::String;

#[cfg(feature = "std")]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::io::{self, IoSlice, IoSliceMut, Read, Write};

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

impl<T: Clone, const N: usize> Clone for StaticVec<T, { N }> {
  #[inline]
  fn clone(&self) -> Self {
    let mut res = Self::new();
    for i in 0..self.length {
      // Safety: `self` has the same capacity as `res`, and `res` is
      // empty, so all of the following writes are safe.
      unsafe {
        //`push_unchecked` (and the previously used iterator) seem to have the same optimizer problem
        //in this "hot loop" context as `filled_with` did generally. So for loop plus `get_unchecked` it is
        //for the time being.
        res
          .data
          .get_unchecked_mut(i)
          .write(self.get_unchecked(i).clone());
        res.length += 1;
      }
    }
    res
  }

  #[inline]
  fn clone_from(&mut self, rhs: &Self) {
    self.truncate(rhs.length);
    for i in 0..self.length {
      // Safety: after the truncate, self.len <= rhs.len, which means that for
      // every i in self, there is definitely an element at rhs[i].
      unsafe {
        self.get_unchecked_mut(i).clone_from(rhs.get_unchecked(i));
      }
    }
    for i in self.length..rhs.length {
      // Safety: i < rhs.length, so rhs.get_unchecked is safe. i starts at
      // self.length, which is <= rhs.length, so there is always an available
      // slot at self[i] to write into.
      unsafe {
        //Same thing with `push_unchecked` here as for `clone`.
        self
          .data
          .get_unchecked_mut(i)
          .write(rhs.get_unchecked(i).clone());
        self.length += 1;
      }
    }
  }
}

impl<T: Debug, const N: usize> Debug for StaticVec<T, { N }> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Debug::fmt(self.as_slice(), f)
  }
}

impl<T, const N: usize> Default for StaticVec<T, { N }> {
  ///Calls `new`.
  #[inline(always)]
  fn default() -> Self {
    Self::new()
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
    self.as_mut_slice()
  }
}

impl<T, const N: usize> Drop for StaticVec<T, { N }> {
  #[inline(always)]
  fn drop(&mut self) {
    unsafe {
      ptr::drop_in_place(self.as_mut_slice());
    }
  }
}

impl<T: Eq, const N: usize> Eq for StaticVec<T, { N }> {}

impl<T, const N: usize> Extend<T> for StaticVec<T, { N }> {
  impl_extend!(val, val, T);
}

#[allow(unused_parens)]
impl<'a, T: 'a + Copy, const N: usize> Extend<&'a T> for StaticVec<T, { N }> {
  impl_extend!(val, (*val), &'a T);
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
  ///[new_from_slice](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &mut [T]) -> Self {
    Self::new_from_slice(values)
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
  ///[new_from_slice](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &[T; N1]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T: Copy, const N1: usize, const N2: usize> From<&mut [T; N1]> for StaticVec<T, { N2 }> {
  ///Creates a new StaticVec instance from the contents of `values`, using
  ///[new_from_slice](crate::StaticVec::new_from_slice) internally.
  #[inline(always)]
  fn from(values: &mut [T; N1]) -> Self {
    Self::new_from_slice(values)
  }
}

impl<T, const N: usize> FromIterator<T> for StaticVec<T, { N }> {
  impl_from_iterator!(val, val, T);
}

#[allow(unused_parens)]
impl<'a, T: 'a + Copy, const N: usize> FromIterator<&'a T> for StaticVec<T, { N }> {
  impl_from_iterator!(val, (*val), &'a T);
}

impl<T: Hash, const N: usize> Hash for StaticVec<T, { N }> {
  #[inline(always)]
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_slice().hash(state);
  }
}

impl<T, I: SliceIndex<[T]>, const N: usize> Index<I> for StaticVec<T, { N }> {
  type Output = I::Output;

  #[inline(always)]
  fn index(&self, index: I) -> &Self::Output {
    self.as_slice().index(index)
  }
}

impl<T, I: SliceIndex<[T]>, const N: usize> IndexMut<I> for StaticVec<T, { N }> {
  #[inline(always)]
  fn index_mut(&mut self, index: I) -> &mut Self::Output {
    self.as_mut_slice().index_mut(index)
  }
}

#[cfg(feature = "std")]
#[doc(cfg(feature = "std"))]
impl<T, const N: usize> Into<Vec<T>> for &mut StaticVec<T, { N }> {
  ///Functionally equivalent to [into_vec](crate::StaticVec::into_vec).
  #[inline(always)]
  fn into(self) -> Vec<T> {
    self.into_vec()
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a StaticVec<T, { N }> {
  type IntoIter = StaticVecIterConst<'a, T>;
  type Item = &'a T;
  ///Returns a `StaticVecIterConst` over the StaticVec's inhabited area.
  #[inline(always)]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a mut StaticVec<T, { N }> {
  type IntoIter = StaticVecIterMut<'a, T>;
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

impl_partial_eq_with_as_slice!(StaticVec<T1, { N1 }>, StaticVec<T2, { N2 }>);
impl_partial_eq_with_as_slice!(StaticVec<T1, { N1 }>, &StaticVec<T2, { N2 }>);
impl_partial_eq_with_as_slice!(StaticVec<T1, { N1 }>, &mut StaticVec<T2, { N2 }>);
impl_partial_eq_with_as_slice!(&StaticVec<T1, { N1 }>, StaticVec<T2, { N2 }>);
impl_partial_eq_with_as_slice!(&mut StaticVec<T1, { N1 }>, StaticVec<T2, { N2 }>);
impl_partial_eq_with_get_unchecked!([T1; N1], StaticVec<T2, { N2 }>);
impl_partial_eq_with_get_unchecked!([T1; N1], &StaticVec<T2, { N2 }>);
impl_partial_eq_with_get_unchecked!([T1; N1], &mut StaticVec<T2, { N2 }>);
impl_partial_eq_with_get_unchecked!(&[T1; N1], StaticVec<T2, { N2 }>);
impl_partial_eq_with_get_unchecked!(&mut [T1; N1], StaticVec<T2, { N2 }>);
impl_partial_eq_with_equals_no_deref!([T1], StaticVec<T2, { N }>);
impl_partial_eq_with_equals_no_deref!([T1], &StaticVec<T2, { N }>);
impl_partial_eq_with_equals_no_deref!([T1], &mut StaticVec<T2, { N }>);
impl_partial_eq_with_equals_deref!(&[T1], StaticVec<T2, { N }>);
impl_partial_eq_with_equals_deref!(&mut [T1], StaticVec<T2, { N }>);
impl_partial_ord_with_as_slice!(StaticVec<T1, { N1 }>, StaticVec<T2, { N2 }>);
impl_partial_ord_with_as_slice!(StaticVec<T1, { N1 }>, &StaticVec<T2, { N2 }>);
impl_partial_ord_with_as_slice!(StaticVec<T1, { N1 }>, &mut StaticVec<T2, { N2 }>);
impl_partial_ord_with_as_slice!(&StaticVec<T1, { N1 }>, StaticVec<T2, { N2 }>);
impl_partial_ord_with_as_slice!(&mut StaticVec<T1, { N1 }>, StaticVec<T2, { N2 }>);
impl_partial_ord_with_get_unchecked!([T1; N1], StaticVec<T2, { N2 }>);
impl_partial_ord_with_get_unchecked!([T1; N1], &StaticVec<T2, { N2 }>);
impl_partial_ord_with_get_unchecked!([T1; N1], &mut StaticVec<T2, { N2 }>);
impl_partial_ord_with_get_unchecked!(&[T1; N1], StaticVec<T2, { N2 }>);
impl_partial_ord_with_get_unchecked!(&mut [T1; N1], StaticVec<T2, { N2 }>);
impl_partial_ord_with_as_slice_against_slice!([T1], StaticVec<T2, { N }>);
impl_partial_ord_with_as_slice_against_slice!([T1], &StaticVec<T2, { N }>);
impl_partial_ord_with_as_slice_against_slice!([T1], &mut StaticVec<T2, { N }>);
impl_partial_ord_with_as_slice_against_slice!(&[T1], StaticVec<T2, { N }>);
impl_partial_ord_with_as_slice_against_slice!(&mut [T1], StaticVec<T2, { N }>);

/// Read from a [`StaticVec`]. This implementation reads from the `StaticVec`
/// by copying bytes into the destination buffers, then shifting the remaining
/// bytes over. This might be inefficient for your needs; consider using
/// [`Cursor`] or [`[T] as Read`][slice-read] for more efficient
/// ways to read out of a `StaticVec` without mutating it.
///
/// [`Cursor`]: https://doc.rust-lang.org/stable/std/io/struct.Cursor.html
/// [slice-read]: https://doc.rust-lang.org/stable/std/primitive.slice.html#impl-Read]
#[cfg(feature = "std")]
impl<const N: usize> Read for StaticVec<u8, { N }> {
  #[inline(always)]
  unsafe fn initializer(&self) -> io::Initializer {
    io::Initializer::nop()
  }

  #[inline]
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let read_length = self.length.min(buf.len());
    // Safety:  read_length <= buf.length and self.length. Rust borrowing
    // rules mean that buf is guaranteed not to overlap with self.
    unsafe {
      buf
        .as_mut_ptr()
        .copy_from_nonoverlapping(self.as_ptr(), read_length);
    }

    if read_length < self.length {
      // TODO: find out if the optimizer elides the bounds check here. It
      // should be able to, since the only non-const value is read_length,
      // which is known to be <= self.length
      self.as_mut_slice().copy_within(read_length.., 0);
    }
    // Safety: 0 <= read_length <= self.length
    self.length -= read_length;
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
    // Minimize copies: copy to each output buf in sequence, then shfit the
    // internal data only once. This as opposed to calling `read` in a loop,
    // which shifts the inner data each time.
    let mut start_ptr = self.as_ptr();
    let original_length = self.length;

    // We update self.length inplace in the loop to track how many bytes
    // have been written. This means that when we perform the shift at the
    // end, self.length is already correct.
    for buf in bufs {
      if self.length == 0 {
        break;
      }

      // The number of bytes we'll be reading out of self
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

    let total_read = original_length - self.length;

    if self.length > 0 {
      // TODO: find out if the optimizer elides the bounds check here. It
      // should be able to, since the only non-const value is total_read,
      // which is known to be <= self.length
      self.as_mut_slice().copy_within(total_read.., 0);
    }

    Ok(total_read)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> Write for StaticVec<u8, { N }> {
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
