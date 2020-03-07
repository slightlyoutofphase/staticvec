#![allow(incomplete_features)]
#![feature(box_syntax)]
#![feature(const_generics)]
#![feature(exact_size_is_empty)]
#![feature(trusted_len)]

// In case you're wondering, the instances of `#[cfg_attr(all(windows, miri), ignore)]` in this
// file above the `#[should_panic]` tests are there simply because Miri only supports catching
// panics on Unix-like OSes and ignores `#[should_panic]` everywhere else, so without the
// configuration attributes those tests just panic normally under Miri on Windows, which we don't
// want.

// A note: This is literally the actual liballoc `BinaryHeap` test suite adapted for `StaticHeap`.

use core::iter::TrustedLen;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU32, Ordering};

use staticvec::*;

type MyStaticVec = StaticVec<i32, 64>;
type MyStaticHeap = StaticHeap<i32, 64>;

#[test]
fn append() {
  let mut a = MyStaticHeap::from(staticvec![-10, 1, 2, 3, 3]);
  let mut b = MyStaticHeap::from(staticvec![-20, 5, 43]);
  a.append(&mut b);
  assert_eq!(a.into_sorted_staticvec(), [-20, -10, 1, 2, 3, 3, 5, 43]);
  assert!(b.is_empty());
}

#[test]
fn append_to_empty() {
  let mut a = StaticHeap::new();
  let mut b = StaticHeap::from(staticvec![-20, 5, 43]);
  a.append(&mut b);
  assert_eq!(a.into_sorted_staticvec(), [-20, 5, 43]);
  assert!(b.is_empty());
}

fn check_exact_size_iterator<I: ExactSizeIterator>(len: usize, it: I) {
  let mut it = it;
  for i in 0..it.len() {
    let (lower, upper) = it.size_hint();
    assert_eq!(Some(lower), upper);
    assert_eq!(lower, len - i);
    assert_eq!(it.len(), len - i);
    it.next();
  }
  assert_eq!(it.len(), 0);
  assert!(it.is_empty());
}

fn check_to_vec<const N: usize>(mut data: StaticVec<i32, N>) {
  let heap = StaticHeap::from(data.clone());
  let mut v = heap.clone().into_staticvec();
  v.sort();
  data.sort();
  assert_eq!(v, data);
  assert_eq!(heap.into_sorted_staticvec(), data);
}

fn check_trusted_len<I: TrustedLen>(len: usize, it: I) {
  let mut it = it;
  for i in 0..len {
    let (lower, upper) = it.size_hint();
    if upper.is_some() {
      assert_eq!(Some(lower), upper);
      assert_eq!(lower, len - i);
    }
    it.next();
  }
}

#[test]
fn drain() {
  let mut q: StaticHeap<i32, 9> = [9, 8, 7, 6, 5, 4, 3, 2, 1].iter().cloned().collect();
  assert_eq!(q.drain().take(5).count(), 5);
  assert!(q.is_empty());
}

#[test]
fn drain_sorted() {
  let mut q: StaticHeap<i32, 9> = [9, 8, 7, 6, 5, 4, 3, 2, 1].iter().cloned().collect();
  assert_eq!(
    q.drain_sorted().take(5).collect::<MyStaticVec>(),
    staticvec![9, 8, 7, 6, 5]
  );
  assert!(q.is_empty());
}

#[test]
fn drain_sorted_collect() {
  let mut heap = StaticHeap::from(staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1]);
  let it = heap.drain_sorted();
  let sorted = it.collect::<MyStaticVec>();
  assert_eq!(sorted, staticvec![10, 9, 8, 7, 6, 5, 4, 3, 2, 2, 1, 1, 0]);
}

#[cfg_attr(all(windows, miri), ignore)]
#[test]
fn drain_sorted_leak() {
  static DROPS: AtomicU32 = AtomicU32::new(0);

  #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
  struct D(u32, bool);

  impl Drop for D {
    fn drop(&mut self) {
      DROPS.fetch_add(1, Ordering::SeqCst);
      if self.1 {
        panic!("panic in `drop`");
      }
    }
  }

  let mut q = StaticHeap::from(staticvec![
    D(0, false),
    D(1, false),
    D(2, false),
    D(3, true),
    D(4, false),
    D(5, false),
  ]);

  catch_unwind(AssertUnwindSafe(|| drop(q.drain_sorted()))).ok();

  assert_eq!(DROPS.load(Ordering::SeqCst), 6);
}

#[test]
fn empty_peek() {
  let empty = StaticHeap::<i32, 0>::new();
  assert!(empty.peek().is_none());
}

#[test]
fn empty_peek_mut() {
  let mut empty = StaticHeap::<i32, 0>::new();
  assert!(empty.peek_mut().is_none());
}

#[test]
fn empty_pop() {
  let mut heap = StaticHeap::<i32, 0>::new();
  assert!(heap.pop().is_none());
}

#[test]
fn exact_size_iterator() {
  let heap = StaticHeap::from(staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1]);
  check_exact_size_iterator(heap.len(), heap.iter());
  check_exact_size_iterator(heap.len(), heap.clone().into_iter());
  check_exact_size_iterator(heap.len(), heap.clone().into_iter_sorted());
  check_exact_size_iterator(heap.len(), heap.clone().drain());
  check_exact_size_iterator(heap.len(), heap.clone().drain_sorted());
}

#[test]
fn extend_ref() {
  let mut a = MyStaticHeap::new();
  a.push(1);
  a.push(2);
  a.extend(&[3, 4, 5]);
  assert_eq!(a.len(), 5);
  assert_eq!(a.into_sorted_staticvec(), [1, 2, 3, 4, 5]);
  let mut a = MyStaticHeap::new();
  a.push(1);
  a.push(2);
  let mut b = MyStaticHeap::new();
  b.push(3);
  b.push(4);
  b.push(5);
  a.extend(b);
  assert_eq!(a.len(), 5);
  assert_eq!(a.into_sorted_staticvec(), [1, 2, 3, 4, 5]);
}

#[test]
fn extend_specialization() {
  let mut a = MyStaticHeap::from(staticvec![-10, 1, 2, 3, 3]);
  let b = MyStaticHeap::from(staticvec![-20, 5, 43]);
  a.extend(b);
  assert_eq!(a.into_sorted_staticvec(), [-20, -10, 1, 2, 3, 3, 5, 43]);
}

#[test]
fn from_iter() {
  let xs = staticvec![9, 8, 7, 6, 5, 4, 3, 2, 1];
  let mut q: StaticHeap<i32, 9> = xs.iter().rev().cloned().collect();
  for &x in &xs {
    assert_eq!(q.pop().unwrap(), x);
  }
}

#[test]
fn is_empty() {
  let a = StaticHeap::<i32, 4>::new();
  assert!(a.is_empty());
}

#[test]
fn is_not_empty() {
  let mut a = StaticHeap::<i32, 4>::new();
  a.push(1);
  assert!(a.is_not_empty());
}

#[test]
fn is_full() {
  let mut a = StaticHeap::<i32, 4>::new();
  a.push(1);
  a.push(2);
  a.push(3);
  a.push(4);
  assert!(a.is_full());
}

#[test]
fn is_not_full() {
  let mut a = StaticHeap::<i32, 4>::new();
  a.push(1);
  a.push(2);
  assert!(a.is_not_full());
}

#[test]
fn iterator() {
  let data = staticvec![5, 9, 3];
  let iterout = [9, 5, 3];
  let heap = StaticHeap::from(data);
  let mut i = 0;
  for el in &heap {
    assert_eq!(*el, iterout[i]);
    i += 1;
  }
}

#[test]
fn iter_rev_cloned_collect() {
  let data = staticvec![5, 9, 3];
  let iterout = staticvec![3, 5, 9];
  let pq = StaticHeap::from(data);
  let v: MyStaticVec = pq.iter().rev().cloned().collect();
  assert_eq!(v, iterout);
}

#[test]
fn into_iter_collect() {
  let data = staticvec![5, 9, 3];
  let iterout = staticvec![9, 5, 3];
  let pq = StaticHeap::from(data);
  let v: MyStaticVec = pq.into_iter().collect();
  assert_eq!(v, iterout);
}

#[test]
fn into_iter_size_hint() {
  let data = staticvec![5, 9];
  let pq = StaticHeap::from(data);
  let mut it = pq.into_iter();
  assert_eq!(it.size_hint(), (2, Some(2)));
  assert_eq!(it.next(), Some(9));
  assert_eq!(it.size_hint(), (1, Some(1)));
  assert_eq!(it.next(), Some(5));
  assert_eq!(it.size_hint(), (0, Some(0)));
  assert_eq!(it.next(), None);
}

#[test]
fn into_iter_rev_collect() {
  let data = staticvec![5, 9, 3];
  let iterout = staticvec![3, 5, 9];
  let pq = StaticHeap::from(data);
  let v: MyStaticVec = pq.into_iter().rev().collect();
  assert_eq!(v, iterout);
}

#[test]
fn into_iter_sorted_collect() {
  let heap = StaticHeap::from(staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1]);
  let it = heap.into_iter_sorted();
  let sorted = it.collect::<MyStaticVec>();
  assert_eq!(sorted, staticvec![10, 9, 8, 7, 6, 5, 4, 3, 2, 2, 1, 1, 0]);
}

// Integrity means that all elements are present after a comparison panics,
// even if the order may not be correct.
//
// Destructors must be called exactly once per element.
#[cfg_attr(all(windows, miri), ignore)]
#[test]
fn panic_safe() {
  use core::cmp;
  use oorandom::Rand32;
  use std::panic::{self, AssertUnwindSafe};
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::time::SystemTime;

  static DROP_COUNTER: AtomicUsize = AtomicUsize::new(0);

  #[derive(Eq, PartialEq, Ord, Clone, Debug)]
  struct PanicOrd<T>(T, bool);

  impl<T> Drop for PanicOrd<T> {
    fn drop(&mut self) {
      // update global drop count
      DROP_COUNTER.fetch_add(1, Ordering::SeqCst);
    }
  }

  impl<T: PartialOrd> PartialOrd for PanicOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
      if self.1 || other.1 {
        panic!("Panicking comparison");
      }
      self.0.partial_cmp(&other.0)
    }
  }

  type PanicVec = StaticVec<PanicOrd<i32>, 64>;

  let mut rng = Rand32::new(
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs(),
  );

  const DATASZ: i32 = 32;
  #[cfg(not(miri))] // Miri is too slow
  const NTEST: i32 = 10;
  #[cfg(miri)]
  const NTEST: i32 = 1;

  // don't use 0 in the data -- we want to catch the zeroed-out case.
  let data = (1..=DATASZ).collect::<MyStaticVec>();

  // since it's a fuzzy test, run several tries.
  for _ in 0..NTEST {
    for i in 1..=DATASZ {
      DROP_COUNTER.store(0, Ordering::SeqCst);

      let mut panic_ords: PanicVec = data
        .iter()
        .filter(|&&x| x != i)
        .map(|&x| PanicOrd(x, false))
        .collect();
      let panic_item = PanicOrd(i, true);

      // heapify the sane items
      for i in (1..panic_ords.len()).rev() {
        panic_ords.swap(i, rng.rand_range(0u32..((i + 1) as u32)) as usize);
      }

      let mut heap = StaticHeap::from(panic_ords);
      let inner_data;

      {
        // push the panicking item to the heap and catch the panic
        let thread_result = {
          let mut heap_ref = AssertUnwindSafe(&mut heap);
          panic::catch_unwind(move || {
            heap_ref.push(panic_item);
          })
        };
        assert!(thread_result.is_err());

        // Assert no elements were dropped
        let drops = DROP_COUNTER.load(Ordering::SeqCst);
        assert!(drops == 0, "Must not drop items. drops={}", drops);
        inner_data = heap.clone().into_staticvec();
        drop(heap);
      }
      let drops = DROP_COUNTER.load(Ordering::SeqCst);
      assert_eq!(drops as i32, DATASZ);

      let mut data_sorted = inner_data.into_iter().map(|p| p.0).collect::<MyStaticVec>();
      data_sorted.sort();
      assert_eq!(data_sorted, data);
    }
  }
}

#[test]
fn peek_and_pop() {
  let data = staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1];
  let mut sorted = data.clone();
  sorted.sort();
  let mut heap = StaticHeap::from(data);
  while !heap.is_empty() {
    assert_eq!(heap.peek().unwrap(), sorted.last().unwrap());
    assert_eq!(heap.pop().unwrap(), sorted.pop().unwrap());
  }
}

#[test]
fn peek_mut() {
  let data = staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1];
  let mut heap = StaticHeap::from(data);
  assert_eq!(heap.peek(), Some(&10));
  {
    let mut top = heap.peek_mut().unwrap();
    *top -= 2;
  }
  assert_eq!(heap.peek(), Some(&9));
}

#[test]
fn peek_mut_pop() {
  let data = staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1];
  let mut heap = StaticHeap::from(data);
  assert_eq!(heap.peek(), Some(&10));
  {
    let mut top = heap.peek_mut().unwrap();
    *top -= 2;
    assert_eq!(StaticHeapPeekMut::pop(top), 8);
  }
  assert_eq!(heap.peek(), Some(&9));
}

#[test]
fn push() {
  let mut heap = MyStaticHeap::from(staticvec![2, 4, 9]);
  assert_eq!(heap.len(), 3);
  assert!(*heap.peek().unwrap() == 9);
  heap.push(11);
  assert_eq!(heap.len(), 4);
  assert!(*heap.peek().unwrap() == 11);
  heap.push(5);
  assert_eq!(heap.len(), 5);
  assert!(*heap.peek().unwrap() == 11);
  heap.push(27);
  assert_eq!(heap.len(), 6);
  assert!(*heap.peek().unwrap() == 27);
  heap.push(3);
  assert_eq!(heap.len(), 7);
  assert!(*heap.peek().unwrap() == 27);
  heap.push(103);
  assert_eq!(heap.len(), 8);
  assert!(*heap.peek().unwrap() == 103);
}

#[test]
fn push_unique() {
  let mut heap = StaticHeap::<Box<_>, 12>::from(staticvec![box 2, box 4, box 9]);
  assert_eq!(heap.len(), 3);
  assert!(**heap.peek().unwrap() == 9);
  heap.push(box 11);
  assert_eq!(heap.len(), 4);
  assert!(**heap.peek().unwrap() == 11);
  heap.push(box 5);
  assert_eq!(heap.len(), 5);
  assert!(**heap.peek().unwrap() == 11);
  heap.push(box 27);
  assert_eq!(heap.len(), 6);
  assert!(**heap.peek().unwrap() == 27);
  heap.push(box 3);
  assert_eq!(heap.len(), 7);
  assert!(**heap.peek().unwrap() == 27);
  heap.push(box 103);
  assert_eq!(heap.len(), 8);
  assert!(**heap.peek().unwrap() == 103);
}

#[test]
fn remaining_capacity() {
  let mut heap = StaticHeap::<i32, 100>::new();
  heap.push(1);
  assert_eq!(heap.remaining_capacity(), 99);
}

#[test]
fn size_in_bytes() {
  let x = StaticHeap::<u8, 8>::from(staticvec![1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(x.size_in_bytes(), 8);
  let y = StaticHeap::<u16, 8>::from(staticvec![1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(y.size_in_bytes(), 16);
  let z = StaticHeap::<u32, 8>::from(staticvec![1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(z.size_in_bytes(), 32);
  let w = StaticHeap::<u64, 8>::from(staticvec![1, 2, 3, 4, 5, 6, 7, 8]);
  assert_eq!(w.size_in_bytes(), 64);
}

#[test]
fn to_vec() {
  check_to_vec(staticvec![]);
  check_to_vec(staticvec![5]);
  check_to_vec(staticvec![3, 2]);
  check_to_vec(staticvec![2, 3]);
  check_to_vec(staticvec![5, 1, 2]);
  check_to_vec(staticvec![1, 100, 2, 3]);
  check_to_vec(staticvec![1, 3, 5, 7, 9, 2, 4, 6, 8, 0]);
  check_to_vec(staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1]);
  check_to_vec(staticvec![
    9, 11, 9, 9, 9, 9, 11, 2, 3, 4, 11, 9, 0, 0, 0, 0
  ]);
  check_to_vec(staticvec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
  check_to_vec(staticvec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
  check_to_vec(staticvec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 1, 2]);
  check_to_vec(staticvec![5, 4, 3, 2, 1, 5, 4, 3, 2, 1, 5, 4, 3, 2, 1]);
}

#[test]
fn trusted_len() {
  let heap = StaticHeap::from(staticvec![2, 4, 6, 2, 1, 8, 10, 3, 5, 7, 0, 9, 1]);
  check_trusted_len(heap.len(), heap.clone().iter());
  check_trusted_len(heap.len(), heap.clone().into_iter());
  check_trusted_len(heap.len(), heap.clone().into_iter_sorted());
  check_trusted_len(heap.len(), heap.clone().drain());
  check_trusted_len(heap.len(), heap.clone().drain_sorted());
}
