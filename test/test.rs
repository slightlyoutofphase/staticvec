use staticvec::*;

#[cfg(feature = "std")]
use std::io::{self, Read, Write};

#[test]
fn as_mut_ptr() {
  let mut v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.as_mut_ptr(), 1) };
}

#[cfg(feature = "std")]
#[test]
fn as_mut_slice() {
  let mut buffer = staticvec![0; 3];
  io::repeat(0b101).read_exact(buffer.as_mut_slice()).unwrap();
}

#[test]
fn as_ptr() {
  let v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.as_ptr(), 1) };
}

#[cfg(feature = "std")]
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
fn clone() {
  let v = staticvec![1, 2, 3, 4, 5, 6, 7, 8];
  let vv = v.clone();
  assert_eq!(v, vv);
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
fn extend() {
  let mut c = StaticVec::<i32, 6>::new();
  c.push(5);
  c.push(6);
  c.push(7);
  c.extend(staticvec![1, 2, 3].iter());
  assert_eq!("[5, 6, 7, 1, 2, 3]", format!("{:?}", c));
  c.clear();
  assert_eq!(c.len(), 0);
  c.extend([1].iter());
  assert_eq!(c.len(), 1);
  c.extend(staticvec![1, 2, 3, 4, 5, 6, 7].iter());
  assert_eq!(c.len(), 6);
}

#[test]
fn extend_from_slice() {
  let mut vec = StaticVec::<i32, 4>::new_from_slice(&[1]);
  vec.extend_from_slice(&[2, 3, 4]);
  assert_eq!(vec, [1, 2, 3, 4]);
}

#[test]
fn filled_with() {
  let mut i = 0;
  let v = StaticVec::<i32, 64>::filled_with(|| {
    i += 1;
    i
  });
  assert_eq!(v.len(), 64);
  assert_eq!(v[0], 1);
  assert_eq!(v[1], 2);
  assert_eq!(v[2], 3);
  assert_eq!(v[3], 4);
}

#[test]
fn from() {
  assert_eq!(
    "[5, 6, 7, 1, 2, 3]",
    format!("{:?}", StaticVec::<i32, 6>::from(&[5, 6, 7, 1, 2, 3]))
  );
  let mut v = staticvec![1];
  v.clear();
  assert_eq!(StaticVec::<i32, 6>::from(v.as_slice()).len(), 0);
}

#[test]
fn index() {
  let vec = staticvec![0, 1, 2, 3, 4];
  assert_eq!(vec[1..4], [1, 2, 3]);
  assert_eq!(vec[1..=1], [1]);
  assert_eq!(vec[1..3], [1, 2]);
  assert_eq!(vec[1..=3], [1, 2, 3]);
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
fn new_from_slice() {
  let vec = StaticVec::<i32, 3>::new_from_slice(&[1, 2, 3]);
  assert_eq!(vec, [1, 2, 3]);
  let vec2 = StaticVec::<i32, 3>::new_from_slice(&[1, 2, 3, 4, 5, 6]);
  assert_eq!(vec2, [1, 2, 3]);
  let vec3 = StaticVec::<i32, 27>::new_from_slice(&[]);
  assert_eq!(vec3, []);
}

#[test]
fn new_from_array() {
  let vec = StaticVec::<i32, 3>::new_from_array([1; 3]);
  assert_eq!(vec, [1, 1, 1]);
  let vec2 = StaticVec::<i32, 3>::new_from_array([1; 6]);
  assert_eq!(vec2, [1, 1, 1]);
  let vec3 = StaticVec::<i32, 27>::new_from_array([0; 0]);
  assert_eq!(vec3, []);
}

#[test]
fn partial_eq() {
  assert_eq!(StaticVec::<i32, 0>::new(), [0; 0]);
  assert_eq!(StaticVec::<i32, 0>::new(), []);
  assert_eq!(StaticVec::<i32, 0>::new(), &[]);
  assert_eq!(StaticVec::<i32, 0>::new(), &mut []);
  assert_eq!(StaticVec::<i32, 0>::new(), StaticVec::<i32, 0>::new());
  assert_eq!(StaticVec::<i32, 0>::new(), &StaticVec::<i32, 0>::new());
  assert_eq!(StaticVec::<i32, 0>::new(), &mut StaticVec::<i32, 0>::new());
  //assert_eq! is written in a way that's limited by LengthAtMost32, so I can't
  //use it for the next part.
  if staticvec![1; 64] != [1; 64] {
    panic!();
  }
  if staticvec![1; 64] != [1; 64] {
    panic!();
  }
  if staticvec![1; 64] != &[1; 64] {
    panic!();
  }
  if staticvec![1; 64] != &mut [1; 64] {
    panic!();
  }
  if staticvec![1; 64] != staticvec![1; 64] {
    panic!();
  }
  if staticvec![1; 64] != &staticvec![1; 64] {
    panic!();
  }
  if staticvec![1; 64] != &mut staticvec![1; 64] {
    panic!();
  }
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
fn reverse() {
  let mut v = staticvec![1, 2, 3];
  v.reverse();
  assert!(v == [3, 2, 1]);
}

#[test]
fn reversed() {
  let v = staticvec![1, 2, 3].reversed();
  assert!(v == [3, 2, 1]);
}

#[test]
fn set_len() {
  let mut v = staticvec![1, 2, 3];
  assert_eq!(v.len(), 3);
  unsafe { v.set_len(0) };
  assert_eq!(v.len(), 0);
}

#[cfg(feature = "std")]
#[test]
fn sort() {
  let mut v = staticvec![-5, 4, 1, -3, 2];
  v.sort();
  assert!(v == [-5, -3, 1, 2, 4]);
}

#[cfg(feature = "std")]
#[test]
fn sorted() {
  let v = staticvec![-5, 4, 1, -3, 2].sorted();
  assert!(v == [-5, -3, 1, 2, 4]);
}

#[test]
fn sort_unstable() {
  let mut v = staticvec![-5, 4, 1, -3, 2];
  v.sort_unstable();
  assert!(v == [-5, -3, 1, 2, 4]);
}

#[test]
fn sorted_unstable() {
  let v = staticvec![-5, 4, 1, -3, 2].sorted_unstable();
  assert!(v == [-5, -3, 1, 2, 4]);
}

#[test]
fn split_off() {
  let mut vec = staticvec![1, 2, 3];
  let vec2 = vec.split_off(1);
  assert_eq!(vec, [1]);
  assert_eq!(vec2, [2, 3]);
}

#[test]
fn swap_pop() {
  let mut v = staticvec!["foo", "bar", "baz", "qux"];
  assert_eq!(v.swap_pop(1).unwrap(), "bar");
  assert_eq!(v, ["foo", "qux", "baz"]);
  assert_eq!(v.swap_pop(0).unwrap(), "foo");
  assert_eq!(v, ["baz", "qux"]);
  assert_eq!(v.swap_pop(17), None);
}

#[test]
fn swap_remove() {
  let mut v = staticvec!["foo", "bar", "baz", "qux"];
  assert_eq!(v.swap_remove(1), "bar");
  assert_eq!(v, ["foo", "qux", "baz"]);
  assert_eq!(v.swap_remove(0), "foo");
  assert_eq!(v, ["baz", "qux"]);
}

#[test]
fn truncate() {
  let mut vec = staticvec![1, 2, 3, 4, 5];
  vec.truncate(2);
  assert_eq!(vec, [1, 2]);
  let mut vec2 = staticvec![1, 2, 3, 4, 5];
  vec2.truncate(2);
  assert_eq!(vec2, [1, 2]);
  let mut vec3 = staticvec![1, 2, 3];
  vec3.truncate(0);
  assert_eq!(vec3, []);
  let mut vec4 = staticvec![1, 2, 3, 4];
  vec4.truncate(97);
  assert_eq!(vec4.len(), 4);
}
