#![allow(clippy::all)]
#![allow(dead_code)]

use staticvec::*;

use core::cell;

#[cfg(not(miri))]
#[cfg(feature = "std")]
use std::panic::{self, AssertUnwindSafe};

#[cfg(feature = "std")]
use std::io::{IoSlice, Write};

#[cfg(not(miri))]
#[cfg(feature = "std")]
use cool_asserts::assert_panics;

#[derive(Debug, Eq, PartialEq, Default)]
struct Counter(cell::Cell<u32>);

impl Counter {
  fn increment(&self) {
    self.0.set(self.0.get() + 1);
  }

  fn get(&self) -> u32 {
    self.0.get()
  }
}

// Helper struct for ensuring things are correctly dropped. Use the `instance`
// method to create a LifespanCountingInstance, then use the init_count
// method to see how many such instances were created (either by clone or by
// `instance`), and the drop_count method to see how many were dropped.
// TODO: create a more advanced version of this pattern that checks WHICH
// elements have been dropped; ie, to ensure that the elements at the end of
// an array are correctly dropped after a truncate
#[derive(Debug, Default)]
struct LifespanCounter {
  // The number of times an instance was created
  init_count: Counter,

  // The number of times an instance was dropped
  drop_count: Counter,
}

impl LifespanCounter {
  fn instance(&self) -> LifespanCountingInstance {
    self.init_count.increment();
    LifespanCountingInstance { counter: self }
  }

  fn init_count(&self) -> u32 {
    self.init_count.get()
  }

  fn drop_count(&self) -> u32 {
    self.drop_count.get()
  }
}

#[derive(Debug)]
struct LifespanCountingInstance<'a> {
  counter: &'a LifespanCounter,
}

impl<'a> Clone for LifespanCountingInstance<'a> {
  fn clone(&self) -> Self {
    self.counter.instance()
  }

  // We deliberately do not provide a clone_from; we'd rather the default
  // behavior (drop and replace with a fresh instance) is used, so we can
  // accurately track clones.
}

impl<'a> Drop for LifespanCountingInstance<'a> {
  fn drop(&mut self) {
    self.counter.drop_count.increment()
  }
}

#[derive(Debug, Eq, PartialEq)]
struct Struct {
  s: &'static str,
}

impl Drop for Struct {
  fn drop(&mut self) {
    // This won't do anything observable in the test context, but it
    // works as a placeholder.
    println!("Dropping Struct with value: {}", self.s)
  }
}

#[test]
fn append() {
  let mut a = staticvec![Struct { s: "A" }, Struct { s: "B" }, Struct { s: "C" }];
  let mut b = staticvec![
    Struct { s: "D" },
    Struct { s: "E" },
    Struct { s: "F" },
    Struct { s: "G" }
  ];
  let mut c = StaticVec::<Struct, 6>::new();
  c.append(&mut a);
  c.append(&mut b);
  assert_eq!(format!("{:?}", a), "[]");
  assert_eq!(format!("{:?}", b), "[Struct { s: \"G\" }]");
  assert_eq!(
    c,
    staticvec![
      Struct { s: "A" },
      Struct { s: "B" },
      Struct { s: "C" },
      Struct { s: "D" },
      Struct { s: "E" },
      Struct { s: "F" }
    ]
  );
  let mut d = staticvec![12, 24];
  let mut e = staticvec![1, 2, 3];
  d.pop().unwrap();
  d.append(&mut e);
  assert_eq!(e, [2, 3]);
  assert_eq!(d, [12, 1]);
}

#[test]
fn as_mut_ptr() {
  let mut v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.as_mut_ptr(), 1) };
}

#[test]
fn as_mut_slice() {
  let mut buffer = staticvec![1, 2, 3, 5, 8];
  assert_eq!(buffer.as_mut_slice(), &mut [1, 2, 3, 5, 8]);
}

#[test]
fn as_ptr() {
  let v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.as_ptr(), 1) };
}

#[test]
fn as_slice() {
  let buffer = staticvec![1, 2, 3, 5, 8];
  assert_eq!(buffer.as_slice(), &[1, 2, 3, 5, 8]);
}

#[cfg(feature = "std")]
#[test]
fn bounds_to_string() {
  let mut v = staticvec![1, 2, 3, 4];
  let it = v.iter();
  assert_eq!(
    "Current value of element at `start`: 1\nCurrent value of element at `end`: 4",
    it.bounds_to_string()
  );
  let itm = v.iter_mut();
  assert_eq!(
    "Current value of element at `start`: 1\nCurrent value of element at `end`: 4",
    itm.bounds_to_string()
  );
  let itv = v.into_iter();
  assert_eq!(
    "Current value of element at `start`: 1\nCurrent value of element at `end`: 4",
    itv.bounds_to_string()
  );
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
fn clone_from_shorter() {
  let src: StaticVec<u32, { 20 }> = (1..10).collect();
  let mut dst: StaticVec<u32, { 20 }> = (0..15).collect();
  dst.clone_from(&src);
  assert_eq!(dst, src);
}

#[test]
fn clone_from_longer() {
  let src: StaticVec<u32, { 20 }> = (0..15).collect();
  let mut dst: StaticVec<u32, { 20 }> = (1..10).collect();
  dst.clone_from(&src);
  assert_eq!(dst, src);
}

#[cfg(not(miri))]
#[cfg(feature = "std")]
#[test]
fn panicking_clone() {
  // An earlier implementation of clone incorrectly leaked values in the event
  // of a panicking clone. This test ensures that that does not happen.
  // This struct will, if so configured, panic on a clone. Uses
  // LifespanCountingInstance to track instantiations and deletions, so that
  // we can ensure the correct number of drops are happening
  #[derive(Debug)]
  struct MaybePanicOnClone<'a> {
    tracker: LifespanCountingInstance<'a>,
    should_panic: bool,
  }

  impl<'a> MaybePanicOnClone<'a> {
    fn new(counter: &'a LifespanCounter, should_panic: bool) -> Self {
      Self {
        tracker: counter.instance(),
        should_panic,
      }
    }
  }

  impl<'a> Clone for MaybePanicOnClone<'a> {
    fn clone(&self) -> Self {
      if self.should_panic {
        panic!("Clone correctly panicked during a test")
      } else {
        Self {
          tracker: self.tracker.clone(),
          should_panic: self.should_panic,
        }
      }
    }
  }

  let lifespan_tracker = LifespanCounter::default();
  let mut vec1: StaticVec<MaybePanicOnClone, 20> = StaticVec::new();

  for _ in 0..5 {
    vec1.push(MaybePanicOnClone::new(&lifespan_tracker, false));
  }
  vec1.push(MaybePanicOnClone::new(&lifespan_tracker, true));

  // Sanity check: we've created 6 instances and dropped none of them
  assert_eq!(lifespan_tracker.init_count(), 6);
  assert_eq!(lifespan_tracker.drop_count(), 0);

  // Attempt to clone the staticvec; this will panic. This should result in
  // 5 successful clones, followed by a panic, followed by 5 drops during
  // unwinding.
  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    let vec2 = vec1.clone();
    vec2
  }));

  // Ensure that a panic did occur
  assert!(result.is_err());

  // At this point, 5 instances should have been created and dropped in the
  // aborted clone
  assert_eq!(lifespan_tracker.init_count(), 11);
  assert_eq!(lifespan_tracker.drop_count(), 5);

  drop(vec1);

  assert_eq!(lifespan_tracker.init_count(), 11);
  assert_eq!(lifespan_tracker.drop_count(), 11);
}

#[test]
fn concat() {
  assert_eq!(
    staticvec!["A, B"].concat(&staticvec!["C", "D", "E", "F"]),
    ["A, B", "C", "D", "E", "F"]
  );
  let v = StaticVec::<i32, 0>::from([]).concat(&StaticVec::<i32, 0>::from([]));
  assert_eq!(v, []);
  let v2 = staticvec![4, 5, 6].concat(&staticvec![1, 2, 3]);
  assert_eq!(v2, [4, 5, 6, 1, 2, 3]);
}

#[test]
fn concat_clone() {
  assert_eq!(
    staticvec![Box::new("A, B")].concat_clone(&staticvec![
      Box::new("C"),
      Box::new("D"),
      Box::new("E"),
      Box::new("F")
    ]),
    [
      Box::new("A, B"),
      Box::new("C"),
      Box::new("D"),
      Box::new("E"),
      Box::new("F")
    ]
  );
  let v = StaticVec::<i32, 0>::from([]).concat_clone(&StaticVec::<i32, 0>::from([]));
  assert_eq!(v, []);
  let v2 = staticvec![Box::new(4), Box::new(5), Box::new(6)].concat_clone(&staticvec![
    Box::new(1),
    Box::new(2),
    Box::new(3)
  ]);
  assert_eq!(
    v2,
    [
      Box::new(4),
      Box::new(5),
      Box::new(6),
      Box::new(1),
      Box::new(2),
      Box::new(3)
    ]
  );
}

#[test]
fn contains() {
  assert_eq!(staticvec![1, 2, 3].contains(&2), true);
  assert_eq!(staticvec![1, 2, 3].contains(&4), false);
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
fn difference() {
  assert_eq!(
    staticvec![4, 5, 6, 7].difference(&staticvec![1, 2, 3, 7]),
    [4, 5, 6]
  );
  assert_eq!(staticvec![1, 2, 3].difference(&staticvec![3, 4, 5]), [1, 2]);
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
fn filled_with_by_index() {
  let v = StaticVec::<usize, 64>::filled_with_by_index(|i| i + 1);
  assert_eq!(v.len(), 64);
  assert_eq!(v[0], 1);
  assert_eq!(v[1], 2);
  assert_eq!(v[2], 3);
  assert_eq!(v[3], 4);
}

#[test]
fn first() {
  let v = staticvec![1, 2, 3];
  assert_eq!(*v.first().unwrap(), 1);
}

#[test]
fn first_mut() {
  let mut v = staticvec![1, 2, 3];
  assert_eq!(*v.first_mut().unwrap(), 1);
}

#[test]
fn from() {
  assert_eq!(
    "[5, 6, 7, 1, 2, 3]",
    format!("{:?}", StaticVec::<i32, 6>::from(&[5, 6, 7, 1, 2, 3]))
  );
  assert_eq!(
    "[1, 1, 1, 1, 1, 1]",
    format!("{:?}", StaticVec::<i32, 6>::from([1; 6]))
  );
  let mut v = staticvec![1];
  v.clear();
  assert_eq!(StaticVec::<i32, 6>::from(v.as_slice()).len(), 0);
  assert_eq!(StaticVec::from(["A"]), ["A"]);
  assert_eq!(
    StaticVec::from([Box::new(Struct { s: "A" }), Box::new(Struct { s: "B" })]),
    [Box::new(Struct { s: "A" }), Box::new(Struct { s: "B" })]
  );
}

#[cfg(feature = "std")]
#[test]
fn from_vec() {
  let v = vec![
    Box::new(Struct { s: "AAA" }),
    Box::new(Struct { s: "BBB" }),
    Box::new(Struct { s: "CCC" }),
  ];
  let vv = StaticVec::<Box<Struct>, 2>::from_vec(v);
  assert_eq!(vv.capacity(), 2);
  assert_eq!(vv.len(), 2);
  assert_eq!(
    vv,
    [Box::new(Struct { s: "AAA" }), Box::new(Struct { s: "BBB" })]
  );
  let x = Vec::<Box<Struct>>::new();
  let y = StaticVec::<Box<Struct>, 1>::from_vec(x);
  assert_eq!(y, []);
}

#[test]
fn get_unchecked() {
  let v = staticvec!["a", "b", "c"];
  assert_eq!(unsafe { *v.get_unchecked(1) }, "b");
}

#[test]
fn get_unchecked_mut() {
  let mut v = staticvec!["a", "b", "c"];
  assert_eq!(unsafe { *v.get_unchecked_mut(1) }, "b");
}

#[test]
fn index() {
  let vec = staticvec![0, 1, 2, 3, 4];
  assert_eq!(vec[3], 3);
  assert_eq!(vec[1..4], [1, 2, 3]);
  assert_eq!(vec[1..=1], [1]);
  assert_eq!(vec[1..3], [1, 2]);
  assert_eq!(vec[..3], [0, 1, 2]);
  assert_eq!(vec[..=3], [0, 1, 2, 3]);
  assert_eq!(vec[1..], [1, 2, 3, 4]);
  assert_eq!(vec[1..=3], [1, 2, 3]);
  assert_eq!(vec[..], [0, 1, 2, 3, 4]);
  // Because this block includes obviously-violated bounds checks, miri
  // complains about it
  #[cfg(not(miri))]
  #[cfg(feature = "std")]
  {
    // Check bounds checking
    assert_panics!(vec[10]);
    assert_panics!(&vec[..10]);
    assert_panics!(&vec[10..]);
    assert_panics!(&vec[10..15]);
    assert_panics!(&vec[1..0]);
  }
}

#[test]
fn insert() {
  let mut vec = StaticVec::<i32, 5>::new_from_slice(&[1, 2, 3]);
  vec.insert(1, 4);
  assert_eq!(vec, [1, 4, 2, 3]);
  vec.insert(4, 5);
  assert_eq!(vec, [1, 4, 2, 3, 5]);
}

// The next few tests for `insert_many` are adapted from the SmallVec testsuite.

#[test]
fn insert_many() {
  let mut v: StaticVec<u8, 8> = StaticVec::new();
  for x in 0..4 {
    v.push(x);
  }
  assert_eq!(v.len(), 4);
  v.insert_many(1, [5, 6].iter().cloned());
  assert_eq!(
    &v.iter().map(|v| *v).collect::<StaticVec<_, 8>>(),
    &[0, 5, 6, 1, 2, 3]
  );
  v.clear();
  for x in 0..4 {
    v.push(x);
  }
  assert_eq!(v.len(), 4);
  v.insert_many(1, [5, 6].iter().cloned());
  assert_eq!(
    &v.iter().map(|v| *v).collect::<StaticVec<_, 8>>(),
    &[0, 5, 6, 1, 2, 3]
  );
  v.clear();
  for i in 0..6 {
    v.push(i + 1);
  }
  v.insert_many(6, [1].iter().cloned());
  assert_eq!(
    &v.iter().map(|v| *v).collect::<StaticVec<_, 8>>(),
    &[1, 2, 3, 4, 5, 6, 1]
  );
}

#[test]
#[should_panic(expected = "Insufficient remaining capacity / out of bounds!")]
fn insert_many_asserts() {
  let mut v: StaticVec<u8, 8> = StaticVec::new();
  for i in 0..7 {
    v.push(i + 1);
  }
  v.insert_many(0, [1, 2, 3, 4].iter().cloned());
  let mut v2: StaticVec<u8, 0> = StaticVec::new();
  v2.insert_many(27, [1, 2, 3, 4].iter().cloned());
}

#[test]
fn intersection() {
  assert_eq!(
    staticvec![4, 5, 6, 7].intersection(&staticvec![1, 2, 3, 7, 4]),
    [4, 7],
  );
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
fn iter() {
  let v = staticvec![1, 2, 3, 4, 5];
  let mut i = v.iter();
  assert_eq!(*i.next().unwrap(), 1);
  assert_eq!(*i.next_back().unwrap(), 5);
  assert_eq!("[2, 3, 4]", format!("{:?}", i));
  assert_eq!(*i.next().unwrap(), 2);
  assert_eq!(*i.next_back().unwrap(), 4);
  assert_eq!("[3]", format!("{:?}", i));
  assert_eq!(*i.next().unwrap(), 3);
  assert_eq!("[]", format!("{:?}", i));
}

#[test]
fn iter_mut() {
  let mut v = staticvec![1, 2, 3, 4, 5];
  let mut i = v.iter_mut();
  assert_eq!(*i.next().unwrap(), 1);
  assert_eq!(*i.next_back().unwrap(), 5);
  assert_eq!("[2, 3, 4]", format!("{:?}", i));
  assert_eq!(*i.next().unwrap(), 2);
  assert_eq!(*i.next_back().unwrap(), 4);
  assert_eq!("[3]", format!("{:?}", i));
  assert_eq!(*i.next().unwrap(), 3);
  assert_eq!("[]", format!("{:?}", i));
}

#[test]
fn into_iter() {
  let v = staticvec![1, 2, 3, 4, 5];
  let mut i = v.into_iter();
  assert_eq!(i.next().unwrap(), 1);
  assert_eq!(i.next_back().unwrap(), 5);
  assert_eq!("[2, 3, 4]", format!("{:?}", i));
  assert_eq!(i.next().unwrap(), 2);
  assert_eq!(i.next_back().unwrap(), 4);
  assert_eq!("[3]", format!("{:?}", i));
  assert_eq!(i.next().unwrap(), 3);
  assert_eq!("[]", format!("{:?}", i));
}

#[cfg(feature = "std")]
#[test]
fn into_vec() {
  let v = staticvec![
    Box::new(Struct { s: "AAA" }),
    Box::new(Struct { s: "BBB" }),
    Box::new(Struct { s: "CCC" })
  ];
  let vv = v.into_vec();
  assert_eq!(vv.capacity(), 3);
  assert_eq!(vv.len(), 3);
}

#[test]
fn last() {
  let v = staticvec![1, 2, 3];
  assert_eq!(*v.last().unwrap(), 3);
}

#[test]
fn last_mut() {
  let mut v = staticvec![1, 2, 3];
  assert_eq!(*v.last_mut().unwrap(), 3);
}

#[test]
fn len() {
  let a = staticvec![1, 2, 3];
  assert_eq!(a.len(), 3);
}

#[test]
fn macros() {
  let v = staticvec![staticvec![staticvec![1, 2, 3, 4]]];
  assert_eq!(v[0][0], [1, 2, 3, 4]);
  let v2 = staticvec![12.0; 64];
  assert!(v2 == [12.0; 64]);
  const V3: StaticVec<i32, 4> = staticvec![1, 2, 3, 4];
  assert_eq!(V3, [1, 2, 3, 4]);
  const V4: StaticVec<i32, 128> = staticvec![27; 128];
  assert!(V4 == [27; 128]);
}

#[test]
fn math_functions() {
  static A: StaticVec<f64, 4> = staticvec![4.0, 5.0, 6.0, 7.0];
  static B: StaticVec<f64, 4> = staticvec![2.0, 3.0, 4.0, 5.0];
  assert_eq!(A.added(&B), [6.0, 8.0, 10.0, 12.0]);
  assert_eq!(A.subtracted(&B), [2.0, 2.0, 2.0, 2.0]);
  assert_eq!(A.multiplied(&B), [8.0, 15.0, 24.0, 35.0]);
  assert_eq!(A.divided(&B), [2.0, 1.6666666666666667, 1.5, 1.4]);
}

#[test]
fn mut_ptr_at() {
  let mut v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.mut_ptr_at(0), 1) };
  unsafe { assert_eq!(*v.mut_ptr_at(1), 2) };
  unsafe { assert_eq!(*v.mut_ptr_at(2), 3) };
}

#[test]
fn mut_ptr_at_unchecked() {
  let mut v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.mut_ptr_at_unchecked(0), 1) };
  unsafe { assert_eq!(*v.mut_ptr_at_unchecked(1), 2) };
  unsafe { assert_eq!(*v.mut_ptr_at_unchecked(2), 3) };
}

#[test]
fn new() {
  let v = StaticVec::<i32, 1>::new();
  assert_eq!(v.capacity(), 1);
}

#[test]
fn new_from_array() {
  let vec = StaticVec::<i32, 3>::new_from_array([1; 3]);
  assert_eq!(vec, [1, 1, 1]);
  let vec2 = StaticVec::<i32, 3>::new_from_array([1; 6]);
  assert_eq!(vec2, [1, 1, 1]);
  let vec3 = StaticVec::<i32, 27>::new_from_array([0; 0]);
  assert_eq!(vec3, []);
  let vec4 = StaticVec::<f32, 1024>::new_from_array([24.0; 512]);
  assert_eq!(vec4, staticvec![24.0; 512]);
  let v = StaticVec::<i32, 3>::new_from_array([1, 2, 3]);
  assert_eq!(v, [1, 2, 3]);
  let v2 = StaticVec::<i32, 3>::new_from_array([1, 2, 3, 4, 5, 6]);
  assert_eq!(v2, [1, 2, 3]);
  let v5 = StaticVec::<Box<Struct>, 2>::new_from_array([
    Box::new(Struct { s: "AAA" }),
    Box::new(Struct { s: "BBB" }),
    Box::new(Struct { s: "CCC" }),
    Box::new(Struct { s: "DDD" }),
    Box::new(Struct { s: "EEE" }),
  ]);
  assert_eq!(
    v5,
    [Box::new(Struct { s: "AAA" }), Box::new(Struct { s: "BBB" })]
  );
}

#[test]
fn new_from_const_array() {
  const VEC2: StaticVec<i32, 6> = StaticVec::new_from_const_array([1; 6]);
  assert_eq!(VEC2, [1, 1, 1, 1, 1, 1]);
  const VEC3: StaticVec<i32, 0> = StaticVec::new_from_const_array([0; 0]);
  assert_eq!(VEC3, []);
  const VEC4: StaticVec<f32, 512> = StaticVec::new_from_const_array([24.0; 512]);
  assert_eq!(VEC4, staticvec![24.0; 512]);
  const V: StaticVec<&'static str, 3> = StaticVec::new_from_const_array(["A", "B", "C"]);
  assert_eq!(V.reversed(), ["C", "B", "A"]);
  const V2: StaticVec<u8, 6> = StaticVec::new_from_const_array([1, 2, 3, 4, 5, 6]);
  assert_eq!(V2, [1, 2, 3, 4, 5, 6]);
  const V6: StaticVec<Struct, 3> = StaticVec::new_from_const_array([
    Struct { s: "AAA" },
    Struct { s: "BBB" },
    Struct { s: "CCC" },
  ]);
  assert_eq!(
    V6,
    [
      Struct { s: "AAA" },
      Struct { s: "BBB" },
      Struct { s: "CCC" },
    ]
  );
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
fn partial_eq() {
  assert_eq!(StaticVec::<i32, 0>::new(), [0; 0]);
  assert_eq!(StaticVec::<i32, 0>::new(), []);
  assert_eq!(StaticVec::<i32, 0>::new(), &[]);
  assert_eq!(StaticVec::<i32, 0>::new(), &mut []);
  assert_eq!(StaticVec::<i32, 0>::new(), StaticVec::<i32, 0>::new());
  assert_eq!(StaticVec::<i32, 0>::new(), &StaticVec::<i32, 0>::new());
  assert_eq!(StaticVec::<i32, 0>::new(), &mut StaticVec::<i32, 0>::new());
  // assert_eq! is written in a way that's limited by LengthAtMost32, so I can't
  // use it for the next part.
  if staticvec![1; 64] != [1; 64] {
    panic!();
  }
  if &staticvec![1; 64] != [1; 64] {
    panic!();
  }
  if &mut staticvec![1; 64] != [1; 64] {
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
fn partial_ord() {
  // TODO: add more here.
  assert!(staticvec![1] < staticvec![2]);
  assert!(staticvec![1] > []);
  assert!(staticvec![1] <= &staticvec![2]);
  assert!(staticvec![1] >= &[]);
  assert!(staticvec![1] > &mut []);
}

#[test]
fn pop() {
  let mut vec = staticvec![1, 2, 3];
  assert_eq!(vec.pop(), Some(3));
  assert_eq!(vec, [1, 2]);
}

#[test]
fn ptr_at() {
  let v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.ptr_at(0), 1) };
  unsafe { assert_eq!(*v.ptr_at(1), 2) };
  unsafe { assert_eq!(*v.ptr_at(2), 3) };
}

#[test]
fn ptr_at_unchecked() {
  let v = staticvec![1, 2, 3];
  unsafe { assert_eq!(*v.ptr_at_unchecked(0), 1) };
  unsafe { assert_eq!(*v.ptr_at_unchecked(1), 2) };
  unsafe { assert_eq!(*v.ptr_at_unchecked(2), 3) };
}

#[test]
fn push() {
  let mut vec = StaticVec::<i32, 4>::new_from_slice(&[1, 2, 3]);
  vec.push(3);
  assert_eq!(vec, [1, 2, 3, 3]);
}

#[cfg(feature = "std")]
mod read_tests {
  use staticvec::*;
  use std::io::{self, Read};
  // We provide custom implementations of most `Read` methods; test those
  // impls
  #[test]
  fn read() {
    let mut ints = staticvec![1, 2, 3, 4, 6, 7, 8, 9, 10];
    let mut buffer = [0, 0, 0, 0];
    assert_eq!(ints.read(&mut buffer).unwrap(), 4);
    assert_eq!(buffer, [1, 2, 3, 4]);
    let mut buffer2 = [];
    assert_eq!(ints.read(&mut buffer2).unwrap(), 0);
    assert_eq!(buffer2, []);
    let mut buffer3 = staticvec![0; 9];
    assert_eq!(ints.read(buffer3.as_mut_slice()).unwrap(), 5);
    assert_eq!(ints, []);
    assert_eq!(ints.read(staticvec![].as_mut_slice()).unwrap(), 0);
  }

  #[test]
  fn read_to_end() {
    let mut ints = staticvec![1, 2, 3, 4, 5, 6, 7];
    let mut buffer = vec![2, 3];
    assert_eq!(ints.read_to_end(&mut buffer).unwrap(), 7);
    assert_eq!(ints, &[]);
    assert_eq!(buffer, &[2, 3, 1, 2, 3, 4, 5, 6, 7]);
  }

  #[test]
  fn read_to_string() {
    // Hello world in ascii
    let mut input = StaticVec::<u8, 30>::new_from_slice(b"World!");
    let mut dest = String::from("Hello, ");
    assert_eq!(input.read_to_string(&mut dest).unwrap(), 6);
    assert_eq!(dest, "Hello, World!");
    assert_eq!(input, &[]);
  }

  #[test]
  fn read_to_string_failure() {
    // Invalid UTF-8 bytes
    let mut input = staticvec![0b1101_1010, 0b1100_0000];
    let mut dest = String::new();
    let err = input.read_to_string(&mut dest).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
  }

  #[test]
  fn read_exact() {
    let mut ints = staticvec![1, 2, 3, 4, 6, 7, 8, 9, 10];
    let mut buffer = [0, 0, 0, 0];
    ints.read_exact(&mut buffer).unwrap();
    assert_eq!(buffer, [1, 2, 3, 4]);
    assert_eq!(ints, &[6, 7, 8, 9, 10]);

    let mut buffer2 = [0, 0, 0, 0, 0, 0, 0, 0];
    let err = ints.read_exact(&mut buffer2).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
  }

  #[test]
  fn read_vectored() {
    let mut ints = staticvec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let mut buf1 = [0; 4];
    let mut buf2 = [0; 4];
    let mut buf3 = [0; 4];
    let bufs = &mut [
      io::IoSliceMut::new(&mut buf1),
      io::IoSliceMut::new(&mut buf2),
      io::IoSliceMut::new(&mut buf3),
    ];
    assert_eq!(ints.read_vectored(bufs).unwrap(), 12);
    assert_eq!(
      "[[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]]",
      format!("{:?}", bufs)
    );
  }
}

#[test]
fn remaining_capacity() {
  let mut v = StaticVec::<i32, 3>::new();
  v.push(12);
  assert_eq!(v.remaining_capacity(), 2);
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
fn reversed() {
  let v = staticvec![1, 2, 3].reversed();
  assert!(v == [3, 2, 1]);
}

#[test]
fn size_in_bytes() {
  let x = StaticVec::<u8, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(x.size_in_bytes(), 8);
  let y = StaticVec::<u16, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(y.size_in_bytes(), 16);
  let z = StaticVec::<u32, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(z.size_in_bytes(), 32);
  let w = StaticVec::<u64, 8>::from([1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(w.size_in_bytes(), 64);
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
fn sorted() {
  let v = staticvec![-5, 4, 1, -3, 2].sorted();
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
fn symmetric_difference() {
  assert_eq!(
    staticvec![1, 2, 3].symmetric_difference(&staticvec![3, 4, 5]),
    [1, 2, 4, 5]
  );
  assert_eq!(
    staticvec![501, 502, 503, 504].symmetric_difference(&staticvec![502, 503, 504, 505]),
    [501, 505]
  );
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
fn triple() {
  static V: StaticVec<usize, 4> = staticvec![4, 5, 6, 7];
  assert_eq!(V.triple(), (V.as_ptr(), 4, 4));
}

#[test]
fn triple_mut() {
  let mut v = staticvec![4, 5, 6, 7];
  let t = v.triple_mut();
  assert_eq!(t, (v.as_mut_ptr(), 4, 4));
  unsafe { *t.0 = 8 };
  assert_eq!(v, [8, 5, 6, 7]);
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

#[test]
fn try_extend_from_slice() {
  let mut v = StaticVec::<i32, 3>::from([1, 2, 3]);
  assert_eq!(v.try_extend_from_slice(&[2, 3]), Err(CapacityError::<3> {}));
  let mut w = StaticVec::<i32, 4>::from([1, 2, 3]);
  assert_eq!(w.try_extend_from_slice(&[2]), Ok(()));
}

#[allow(unused_must_use)]
#[test]
fn try_insert() {
  let mut vec = staticvec![1, 2, 3, 4, 5];
  assert_eq!(vec.try_insert(2, 0), Err(CapacityError::<5> {}));
  let mut vec2 = StaticVec::<i32, 4>::new_from_slice(&[1, 2, 3]);
  vec2.try_insert(2, 3);
  assert_eq!(vec2, [1, 2, 3, 3]);
}

#[test]
fn try_push() {
  let mut vec = staticvec![1, 2, 3, 4, 5];
  let err = vec.try_push(2).unwrap_err();
  assert_eq!(err.into_value(), 2);
  let mut vec2 = StaticVec::<i32, 4>::new_from_slice(&[1, 2, 3]);
  assert_eq!(vec2.try_push(3), Ok(()));
  assert_eq!(vec2, [1, 2, 3, 3]);
}

#[cfg(feature = "std")]
#[test]
fn write() {
  // From arrayvec
  let mut v = StaticVec::<u8, 8>::new();
  write!(&mut v, "\x01\x02\x03").unwrap();
  assert_eq!(&v[..], &[1, 2, 3]);
  let r = v.write(&[9; 16]).unwrap();
  assert_eq!(r, 5);
  assert_eq!(&v[..], &[1, 2, 3, 9, 9, 9, 9, 9]);
}

#[cfg(feature = "std")]
#[test]
fn write_all() {
  let mut v = StaticVec::<u8, 6>::new();
  assert!(v.write_all(&[1, 2, 3, 4, 5, 6, 7, 8]).is_err());
  v.clear();
  assert!(v.write_all(&[1, 2, 3, 4, 5, 6]).is_ok());
}

#[cfg(feature = "std")]
#[test]
fn write_vectored() {
  let mut v = StaticVec::<u8, 8>::new();
  assert_eq!(
    v.write_vectored(&[IoSlice::new(&[1, 2, 3, 4]), IoSlice::new(&[5, 6, 7, 8])])
      .unwrap(),
    8
  );
  assert_eq!(v, [1, 2, 3, 4, 5, 6, 7, 8]);
}
