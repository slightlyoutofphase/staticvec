use staticvec::*;
use std::io::{self, Read, Write};

#[test]
fn as_mut_slice() {
  let mut buffer = staticvec![0; 3];
  io::repeat(0b101).read_exact(buffer.as_mut_slice()).unwrap();
}

#[test]
fn as_slice() {
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
  assert_eq!(vec, [1, 2, 3, 2]);
}

#[test]
fn dedup_by() {
  let mut vec = staticvec!["foo", "bar", "Bar", "baz", "bar"];
  vec.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
  assert_eq!(vec, ["foo", "bar", "baz", "bar"]);
}

#[test]
fn dedup_by_key() {
  let mut vec = staticvec![10, 20, 21, 30, 20];
  vec.dedup_by_key(|i| *i / 10);
  assert_eq!(vec, [10, 20, 30, 20]);
}

#[test]
fn drain() {
  let mut v = staticvec![1, 2, 3];
  let u = v.drain(1..);
  assert_eq!(v, &[1]);
  assert_eq!(u, &[2, 3]);
  v.drain(..);
  assert_eq!(v, &[]);
}

#[test]
fn drain_filter() {
  let mut numbers = staticvec![1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15];
  let evens = numbers.drain_filter(|x| *x % 2 == 0);
  let odds = numbers;
  assert_eq!(evens, [2, 4, 6, 8, 14]);
  assert_eq!(odds, [1, 3, 5, 9, 11, 13, 15]);
}

#[test]
fn extend_from_slice() {
  let mut vec = StaticVec::<i32, 4>::new_from_slice(&[1]);
  vec.extend_from_slice(&[2, 3, 4]);
  assert_eq!(vec, [1, 2, 3, 4]);
}

#[test]
fn insert() {
  let mut vec = StaticVec::<i32, 5>::new_from_slice(&[1, 2, 3]);
  vec.insert(1, 4);
  assert_eq!(vec, [1, 4, 2, 3]);
  vec.insert(4, 5);
  assert_eq!(vec, [1, 4, 2, 3, 5]);
}

#[test]
fn is_empty() {
  let mut v = StaticVec::<i32, 1>::new();
  assert!(v.is_empty());
  v.push(1);
  assert!(!v.is_empty());
}

#[test]
fn is_not_empty() {
  let mut v = StaticVec::<i32, 1>::new();
  assert!(v.is_empty());
  v.push(1);
  assert!(v.is_not_empty());
}

#[test]
fn is_full() {
  let mut v = StaticVec::<i32, 1>::new();
  v.push(1);
  assert!(v.is_full());
}

#[test]
fn is_not_full() {
  let v = StaticVec::<i32, 1>::new();
  assert!(v.is_not_full());
}

#[test]
fn len() {
  let a = staticvec![1, 2, 3];
  assert_eq!(a.len(), 3);
}

#[test]
fn new() {
  let v = StaticVec::<i32, 1>::new();
  assert_eq!(v.capacity(), 1);
}

#[test]
fn pop() {
  let mut vec = staticvec![1, 2, 3];
  assert_eq!(vec.pop(), Some(3));
  assert_eq!(vec, [1, 2]);
}

#[test]
fn push() {
  let mut vec = StaticVec::<i32, 4>::new_from_slice(&[1, 2, 3]);
  vec.push(3);
  assert_eq!(vec, [1, 2, 3, 3]);
}

#[test]
fn remove() {
  let mut v = staticvec![1, 2, 3];
  assert_eq!(v.remove(1), 2);
  assert_eq!(v, [1, 3]);
}

#[test]
fn remove_item() {
  let mut vec = staticvec![1, 2, 3, 1];
  vec.remove_item(&1);
  assert_eq!(vec, staticvec![2, 3, 1]);
}

#[test]
fn retain() {
  let mut vec = staticvec![1, 2, 3, 4, 5];
  let keep = [false, true, true, false, true];
  let mut i = 0;
  vec.retain(|_| (keep[i], i += 1).0);
  assert_eq!(vec, [2, 3, 5]);
}

#[test]
fn set_len() {
  let mut v = staticvec![1, 2, 3];
  assert_eq!(v.len(), 3);
  unsafe { v.set_len(0) };
  assert_eq!(v.len(), 0);
}

#[test]
fn split_off() {
  let mut vec = staticvec![1, 2, 3];
  let vec2 = vec.split_off(1);
  assert_eq!(vec, [1]);
  assert_eq!(vec2, [2, 3]);
}
