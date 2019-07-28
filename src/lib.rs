#![feature(const_fn)]
#![feature(const_generics)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_ref)]

use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};

///A Vec-like struct (directly API-compatible where it can be) implemented with
///const generics around a static array of fixed "N" capacity.
pub struct StaticVec<T, const N: usize> {
  data: [MaybeUninit<T>; N],
  length: usize,
}

///Vaguely similar to a very stripped-down version of std::slice::Iter.
pub struct StaticVecIteratorConst<'a, T: 'a> {
  current: *const T,
  end: *const T,
  marker: PhantomData<&'a T>,
}

///Vaguely similar to a very stripped-down version of std::slice::IterMut.
pub struct StaticVecIteratorMut<'a, T: 'a> {
  current: *mut T,
  end: *mut T,
  marker: PhantomData<&'a mut T>,
}

impl<T, const N: usize> StaticVec<T, { N }> {
  ///Returns a new StaticVec instance, after asserting that "N" is greater than 0.
  pub fn new() -> Self {
    unsafe {
      Self {
        data: MaybeUninit::uninit().assume_init(),
        length: 0,
      }
    }
  }

  ///Returns the current length of the StaticVec.
  ///Just as for a normal Vec, this means the number of elements that
  ///have been added to it with "push", "insert", e.t.c.
  pub fn len(&self) -> usize {
    self.length
  }

  ///Returns the total capacity of the StaticVec.
  ///This is always equivalent to the generic "N" parameter it was declared with,
  ///which determines the fixed size of the static backing array.
  pub const fn capacity(&self) -> usize {
    N
  }

  ///Returns true if the current length of the StaticVec is 0.
  pub fn is_empty(&self) -> bool {
    self.length == 0
  }

  ///Returns true if the current length of the StaticVec is greater than 0.
  pub fn is_not_empty(&self) -> bool {
    self.length > 0
  }

  ///Returns true if the current length of the StaticVec is equal to its capacity.
  pub fn is_full(&self) -> bool {
    self.length == N
  }

  ///Returns true if the current length of the StaticVec is less than its capacity.
  pub fn is_not_full(&self) -> bool {
    self.length < N
  }

  ///Returns a constant pointer to the first element of the StaticVec's internal array.
  pub fn as_ptr(&self) -> *const T {
    self.data.as_ptr() as *const T
  }

  ///Returns a mutable pointer to the first element of the StaticVec's internal array.
  pub fn as_mut_ptr(&mut self) -> *mut T {
    self.data.as_mut_ptr() as *mut T
  }

  ///Returns a constant reference to a slice of the StaticVec's "inhabited" area.
  pub fn as_slice(&self) -> &[T] {
    unsafe {
      (&self.data[0..self.length] as *const [MaybeUninit<T>] as *const [T])
        .as_ref()
        .unwrap()
    }
  }

  ///Returns a mutable reference to a slice of the StaticVec's "inhabited" area.
  pub fn as_mut_slice(&mut self) -> &mut [T] {
    unsafe {
      (&mut self.data[0..self.length] as *mut [MaybeUninit<T>] as *mut [T])
        .as_mut()
        .unwrap()
    }
  }

  ///Asserts that the current length of the StaticVec is less than "N",
  ///and if so appends a value to the end of it.
  pub fn push(&mut self, value: T) {
    assert!(self.length < N, "No space left!");
    unsafe { self.data.get_unchecked_mut(self.length).write(value) };
    self.length += 1;
  }

  ///Removes the value at the last position of the StaticVec and returns it in "Some" if
  ///the StaticVec has a current length greater than 0, and "None" otherwise.
  pub fn pop(&mut self) -> Option<T> {
    if self.length == 0 {
      None
    } else {
      self.length -= 1;
      unsafe { Some(self.data.get_unchecked(self.length).read()) }
    }
  }

  ///Appends a value to the end of the StaticVec without asserting that
  ///its current length is less than "N".
  pub unsafe fn push_unchecked(&mut self, value: T) {
    self.data.get_unchecked_mut(self.length).write(value);
    self.length += 1;
  }

  ///Pops a value from the end of the StaticVec and returns it directly without asserting that
  ///the StaticVec's current length is greater than 0.
  pub unsafe fn pop_unchecked(&mut self) -> T {
    self.length -= 1;
    self.data.get_unchecked(self.length).read()
  }

  ///Asserts that "index" is less than the current length of the StaticVec,
  ///and if so removes the value at that position and returns it. Any values
  ///that exist in later positions are shifted to the left.
  pub fn remove(&mut self, index: usize) -> T {
    assert!(index < self.length, "Out of range!");
    unsafe {
      let p = self.as_mut_ptr().add(index);
      let res = p.read();
      p.offset(1).copy_to(p, self.length - index - 1);
      self.length -= 1;
      res
    }
  }

  ///Asserts that the current length of the StaticVec is less than "N" and that
  ///"index" is less than the length, and if so inserts "value" at that position.
  ///Any values that exist in later positions are shifted to the right.
  pub fn insert(&mut self, index: usize, value: T) {
    assert!(
      self.length < N && index <= self.length,
      "Either you're out of range or there's no space left!"
    );
    unsafe {
      let p = self.as_mut_ptr().add(index);
      p.copy_to(p.offset(1), self.length - index);
      p.write(value);
      self.length += 1;
    }
  }

  ///Removes all contents from the StaticVec and sets it length back to 0.
  pub fn clear(&mut self) {
    unsafe {
      for i in 0..self.length {
        *self.data.get_unchecked_mut(i) = MaybeUninit::uninit();
      }
    }
    self.length = 0;
  }

  ///Performs an in-place sort of the StaticVec's "inhabited" area.
  pub fn sort(&mut self)
  where T: std::cmp::Ord {
    self.as_mut_slice().sort();
  }

  ///Reverses the contents of the StaticVec's "inhabited" area in-place.
  pub fn reverse(&mut self) {
    self.as_mut_slice().reverse();
  }

  ///Returns a separate, sorted StaticVec of the contents of the StaticVec's "inhabited" area without modifying
  ///the original data.
  pub fn sorted(&mut self) -> Self
  where T: std::cmp::Ord {
    unsafe {
      let mut res = Self::new();
      res.length = self.length;
      self.as_ptr().copy_to(res.as_mut_ptr(), self.length);
      res.sort();
      res
    }
  }

  ///Returns a separate, reversed StaticVec of the contents of the StaticVec's "inhabited" area without modifying
  ///the original data.
  pub fn reversed(&mut self) -> Self {
    unsafe {
      let mut res = Self::new();
      res.length = self.length;
      self.as_ptr().copy_to(res.as_mut_ptr(), self.length);
      res.reverse();
      res
    }
  }

  ///Returns a StaticVecIteratorConst over the StaticVec's "inhabited" area.
  pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
    unsafe {
      if self.length > 0 {
        StaticVecIteratorConst::<'a, T> {
          current: self.as_ptr(),
          end: self.as_ptr().add(self.length),
          marker: PhantomData,
        }
      } else {
        StaticVecIteratorConst::<'a, T> {
          current: self.as_ptr(),
          end: self.as_ptr(),
          marker: PhantomData,
        }
      }
    }
  }

  ///Returns a StaticVecIteratorMut over the StaticVec's "inhabited" area.
  pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> {
    unsafe {
      if self.length > 0 {
        StaticVecIteratorMut::<'a, T> {
          current: self.as_mut_ptr(),
          end: self.as_mut_ptr().add(self.length),
          marker: PhantomData,
        }
      } else {
        StaticVecIteratorMut::<'a, T> {
          current: self.as_mut_ptr(),
          end: self.as_mut_ptr(),
          marker: PhantomData,
        }
      }
    }
  }
}

impl<T, const N: usize> Index<usize> for StaticVec<T, { N }> {
  type Output = T;
  ///Asserts that "index" is less than the current length of the StaticVec,
  ///as if so returns the value at that position as a constant reference.
  fn index(&self, index: usize) -> &Self::Output {
    assert!(index < self.length, "Out of range!");
    unsafe { self.data.get_unchecked(index).get_ref() }
  }
}

impl<T, const N: usize> IndexMut<usize> for StaticVec<T, { N }> {
  ///Asserts that "index" is less than the current length of the StaticVec,
  ///as if so returns the value at that position as a mutable reference.
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    assert!(index < self.length, "Out of range!");
    unsafe { self.data.get_unchecked_mut(index).get_mut() }
  }
}

impl<'a, T: 'a> Iterator for StaticVecIteratorConst<'a, T> {
  type Item = &'a T;
  ///Returns "Some(self.current.as_ref().unwrap())" if "current" is less than or equal to "end",
  ///and None if it's not.
  fn next(&mut self) -> Option<Self::Item> {
    if self.current < self.end {
      unsafe {
        let res = Some(self.current.as_ref().unwrap());
        self.current = self.current.add(1);
        res
      }
    } else {
      None
    }
  }
}

impl<'a, T: 'a> Iterator for StaticVecIteratorMut<'a, T> {
  type Item = &'a mut T;
  ///Returns "Some(self.current.as_mut().unwrap())" if "current" is less than or equal to "end",
  ///and None if it's not.
  fn next(&mut self) -> Option<Self::Item> {
    if self.current < self.end {
      unsafe {
        let res = Some(self.current.as_mut().unwrap());
        self.current = self.current.add(1);
        res
      }
    } else {
      None
    }
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a StaticVec<T, { N }> {
  type IntoIter = StaticVecIteratorConst<'a, T>;
  type Item = <Self::IntoIter as Iterator>::Item;
  ///Returns a StaticVecIteratorConst over the StaticVec's "inhabited" area.
  fn into_iter(self) -> Self::IntoIter {
    unsafe {
      if self.length > 0 {
        Self::IntoIter {
          current: self.as_ptr(),
          end: self.as_ptr().add(self.length),
          marker: PhantomData,
        }
      } else {
        Self::IntoIter {
          current: self.as_ptr(),
          end: self.as_ptr(),
          marker: PhantomData,
        }
      }
    }
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a mut StaticVec<T, { N }> {
  type IntoIter = StaticVecIteratorMut<'a, T>;
  type Item = <Self::IntoIter as Iterator>::Item;
  ///Returns a StaticVecIteratorMut over the StaticVec's "inhabited" area.
  fn into_iter(self) -> Self::IntoIter {
    unsafe {
      if self.length > 0 {
        Self::IntoIter {
          current: self.as_mut_ptr(),
          end: self.as_mut_ptr().add(self.length),
          marker: PhantomData,
        }
      } else {
        Self::IntoIter {
          current: self.as_mut_ptr(),
          end: self.as_mut_ptr(),
          marker: PhantomData,
        }
      }
    }
  }
}

impl<T, const N: usize> FromIterator<T> for StaticVec<T, { N }> {
  ///Attempts to create a new StaticVec instance of the specified capacity from "iter".
  ///If it has a size greater than the capacity, any contents after that point are ignored.
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut res = StaticVec::<T, { N }>::new();
    for value in iter {
      if res.is_full() {
        break;
      } else {
        res.push(value);
      }
    }
    res
  }
}
