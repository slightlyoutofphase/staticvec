use staticvec::*;

#[test]
fn as_mut_slice() {
  use std::io::{self, Read};
  let mut buffer = staticvec![0; 3];
  io::repeat(0b101).read_exact(buffer.as_mut_slice()).unwrap();
}

#[test]
fn as_slice() {
  use std::io::{self, Write};
  let buffer = staticvec![1, 2, 3, 5, 8];
  io::sink().write(buffer.as_slice()).unwrap();
}

#[test]
fn capacity() {
  let vec = StaticVec::<i32, 10>::new();
  assert_eq!(vec.capacity(), 10);
}

#[test]
fn clear() {
  let mut v = staticvec![1, 2, 3];
  v.clear();
  assert!(v.is_empty());
}

#[test]
fn dedup() {
  let mut vec = staticvec![1, 2, 2, 3, 2];
  vec.dedup();
  assert_eq!(vec.as_slice(), [1, 2, 3, 2]);
}

#[test]
fn dedup_by() {
  let mut vec = staticvec!["foo", "bar", "Bar", "baz", "bar"];
  vec.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
  assert_eq!(vec.as_slice(), ["foo", "bar", "baz", "bar"]);
}

#[test]
fn dedup_by_key() {
  let mut vec = staticvec![10, 20, 21, 30, 20];
  vec.dedup_by_key(|i| *i / 10);
  assert_eq!(vec.as_slice(), [10, 20, 30, 20]);
}

#[test]
fn drain() {
  let mut v = staticvec![1, 2, 3];
  let u = v.drain(1..);
  assert_eq!(v.as_slice(), &[1]);
  assert_eq!(u.as_slice(), &[2, 3]);
  v.drain(..);
  assert_eq!(v.as_slice(), &[]);
}

#[test]
fn drain_filter() {
  let mut numbers = staticvec![1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15];
  let evens = numbers.drain_filter(|x| *x % 2 == 0);
  let odds = numbers;
  assert_eq!(evens.as_slice(), [2, 4, 6, 8, 14]);
  assert_eq!(odds.as_slice(), [1, 3, 5, 9, 11, 13, 15]);
}
