use super::{StaticHeap, StaticHeapIntoIter, StaticHeapIter};
use crate::trait_impls::ExtendEx;
use crate::StaticVec;
use core::fmt::{self, Debug, Formatter};
use core::iter::FromIterator;

impl<T: Clone, const N: usize> Clone for StaticHeap<T, N> {
  #[inline(always)]
  default fn clone(&self) -> Self {
    StaticHeap {
      data: self.data.clone(),
    }
  }

  #[inline(always)]
  default fn clone_from(&mut self, source: &Self) {
    self.data.clone_from(&source.data);
  }
}

impl<T: Copy, const N: usize> Clone for StaticHeap<T, N> {
  #[inline(always)]
  fn clone(&self) -> Self {
    StaticHeap {
      data: self.data.clone(),
    }
  }

  #[inline(always)]
  fn clone_from(&mut self, source: &Self) {
    self.data.clone_from(&source.data);
  }
}

impl<T: Ord, const N: usize> Default for StaticHeap<T, N> {
  /// Creates an empty `StaticHeap<T, N>`.
  #[inline(always)]
  fn default() -> StaticHeap<T, N> {
    StaticHeap::new()
  }
}

impl<T: Debug, const N: usize> Debug for StaticHeap<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_list().entries(self.iter()).finish()
  }
}

impl<T: Ord, I: IntoIterator<Item = T>, const N: usize> ExtendEx<T, I> for StaticHeap<T, N> {
  #[inline(always)]
  default fn extend_ex(&mut self, iter: I) {
    self.data.extend(iter);
    self.rebuild();
  }

  #[inline(always)]
  default fn from_iter_ex(iter: I) -> Self {
    StaticHeap::from(StaticVec::from_iter(iter))
  }
}

impl<'a, T: 'a + Copy + Ord, I: IntoIterator<Item = &'a T>, const N: usize> ExtendEx<&'a T, I>
  for StaticHeap<T, N>
{
  #[inline(always)]
  default fn extend_ex(&mut self, iter: I) {
    self.data.extend(iter);
    self.rebuild();
  }

  #[inline(always)]
  default fn from_iter_ex(iter: I) -> Self {
    StaticHeap::from(StaticVec::from_iter(iter))
  }
}

impl<T: Ord, const N: usize> Extend<T> for StaticHeap<T, N> {
  #[inline(always)]
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    <Self as ExtendEx<T, I>>::extend_ex(self, iter);
  }
}

impl<'a, T: 'a + Copy + Ord, const N: usize> Extend<&'a T> for StaticHeap<T, N> {
  #[inline(always)]
  fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
    <Self as ExtendEx<&'a T, I>>::extend_ex(self, iter);
  }
}

impl<T: Ord, const N: usize> ExtendEx<T, StaticHeap<T, N>> for StaticHeap<T, N> {
  #[inline(always)]
  fn extend_ex(&mut self, ref mut other: StaticHeap<T, N>) {
    self.append(other);
  }

  #[inline(always)]
  fn from_iter_ex(iter: StaticHeap<T, N>) -> Self {
    StaticHeap::from(iter.into_iter().collect::<StaticVec<_, N>>())
  }
}

impl<T: Ord, const N1: usize, const N2: usize> From<StaticVec<T, N1>> for StaticHeap<T, N2> {
  /// Converts a `StaticVec<T, N>` into a `StaticHeap<T, N>`.
  /// This conversion happens in-place, and has `O(n)` time complexity.
  #[inline(always)]
  default fn from(vec: StaticVec<T, N1>) -> StaticHeap<T, N2> {
    let mut heap = StaticHeap {
      data: StaticVec::from_iter(vec.into_iter()),
    };
    heap.rebuild();
    heap
  }
}

impl<T: Ord, const N: usize> From<StaticVec<T, N>> for StaticHeap<T, N> {
  /// Converts a `StaticVec<T, N>` into a `StaticHeap<T, N>`.
  /// This conversion happens in-place, and has `O(n)` time complexity.
  #[inline(always)]
  fn from(vec: StaticVec<T, N>) -> StaticHeap<T, N> {
    let mut heap = StaticHeap { data: vec };
    heap.rebuild();
    heap
  }
}

impl<T: Ord, const N: usize> FromIterator<T> for StaticHeap<T, N> {
  #[inline(always)]
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> StaticHeap<T, N> {
    <Self as ExtendEx<T, I>>::from_iter_ex(iter)
  }
}

impl<'a, T: 'a + Copy + Ord, const N: usize> FromIterator<&'a T> for StaticHeap<T, N> {
  #[inline(always)]
  fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> StaticHeap<T, N> {
    <Self as ExtendEx<&'a T, I>>::from_iter_ex(iter)
  }
}

impl<T, const N: usize> IntoIterator for StaticHeap<T, N> {
  type Item = T;
  type IntoIter = StaticHeapIntoIter<T, N>;

  /// Creates a consuming iterator, that is, one that moves each value out of
  /// the binary heap in arbitrary order. The binary heap cannot be used
  /// after calling this.
  ///
  /// # Examples
  ///
  /// Basic usage:
  /// ```
  /// # use staticvec::*;
  /// let heap = StaticHeap::from(staticvec![1, 2, 3, 4]);
  /// // Print 1, 2, 3, 4 in arbitrary order
  /// for x in heap.into_iter() {
  ///   // x has type i32, not &i32
  ///   println!("{}", x);
  /// }
  /// ```
  #[inline(always)]
  fn into_iter(self) -> StaticHeapIntoIter<T, N> {
    StaticHeapIntoIter {
      iter: self.data.into_iter(),
    }
  }
}

impl<'a, T, const N: usize> IntoIterator for &'a StaticHeap<T, N> {
  type Item = &'a T;
  type IntoIter = StaticHeapIter<'a, T, N>;

  #[inline(always)]
  fn into_iter(self) -> StaticHeapIter<'a, T, N> {
    StaticHeapIter { iter: self.iter() }
  }
}
