#![allow(incomplete_features)]
#![allow(clippy::all)]
#![feature(test)]
#![feature(const_generics)]

//SlightlyOutOfPhase, August 2019:

//This is, obviously, a modified version of SmallVec's benchmark suite.
//The main difference is that while the original was a lot about capacity-increase performance,
//that's not relevant for StaticVec, and so instead I compare it moreso against Vecs constructed
//via the `with_capacity` function in order to keep things as equal as possible.

use staticvec::*;

extern crate test;

use test::{Bencher, black_box};

const VEC_SIZE: usize = 16;
const SPILLED_SIZE: usize = 100;

//Needed to not crash the insert benchmark since capacity can't change for StaticVec.
const SPILLED_SIZE_TWO: usize = 98;

pub trait ExtendFromSlice<T> {
  fn extend_from_slice(&mut self, other: &[T]);
}

impl<T: Copy> ExtendFromSlice<T> for Vec<T> {
  fn extend_from_slice(&mut self, other: &[T]) {
    Vec::extend_from_slice(self, other)
  }
}

impl<T: Copy, const N: usize> ExtendFromSlice<T> for StaticVec<T, { N }> {
  fn extend_from_slice(&mut self, other: &[T]) {
    StaticVec::<T, { N }>::extend_from_slice(self, other)
  }
}

trait Vector<T>: for<'a> From<&'a [T]> + Extend<T> + ExtendFromSlice<T> {
  fn new() -> Self;
  fn push(&mut self, val: T);
  fn pop(&mut self) -> Option<T>;
  fn remove(&mut self, p: usize) -> T;
  fn insert(&mut self, n: usize, val: T);
  fn from_elem(val: T, n: usize) -> Self;
  fn from_elems(val: &[T]) -> Self;
}

impl<T: Copy + 'static> Vector<T> for Vec<T> {
  fn new() -> Self {
    //SlightlyOutOfPhase, August 2019:

    //StaticVec can't start with a capacity of VEC_SIZE and grow to SPILLED_SIZE,
    //so I start both normal Vec and StaticVec with a capacity of SPILLED_SIZE.
    Self::with_capacity(SPILLED_SIZE)
  }

  fn push(&mut self, val: T) {
    self.push(val)
  }

  fn pop(&mut self) -> Option<T> {
    self.pop()
  }

  fn remove(&mut self, p: usize) -> T {
    self.remove(p)
  }

  fn insert(&mut self, n: usize, val: T) {
    self.insert(n, val)
  }

  fn from_elem(val: T, n: usize) -> Self {
    vec![val; n]
  }

  fn from_elems(val: &[T]) -> Self {
    val.to_owned()
  }
}

impl<T: Copy + 'static, const N: usize> Vector<T> for StaticVec<T, { N }> {
  fn new() -> Self {
    Self::new()
  }

  fn push(&mut self, val: T) {
    self.push(val)
  }

  fn pop(&mut self) -> Option<T> {
    self.pop()
  }

  fn remove(&mut self, p: usize) -> T {
    self.remove(p)
  }

  fn insert(&mut self, n: usize, val: T) {
    self.insert(n, val)
  }

  fn from_elem(val: T, _n: usize) -> Self {
    staticvec![val; {N}]
  }

  fn from_elems(val: &[T]) -> Self {
    Self::new_from_slice(val)
  }
}

macro_rules! make_benches {
    ($typ:ty { $($b_name:ident => $g_name:ident($($args:expr),*),)* }) => {
        $(
            #[bench]
            fn $b_name(b: &mut Bencher) {
                $g_name::<$typ>($($args,)* b)
            }
        )*
    }
}

make_benches! {
  StaticVec<u64, {SPILLED_SIZE}> {
    staticvec_bench_push => gen_push(SPILLED_SIZE as _),
    staticvec_bench_push_small => gen_push(VEC_SIZE as _),
    staticvec_bench_insert => gen_insert(SPILLED_SIZE_TWO as _),
    staticvec_bench_insert_small => gen_insert(VEC_SIZE as _),
    staticvec_bench_remove => gen_remove(SPILLED_SIZE as _),
    staticvec_bench_remove_small => gen_remove(VEC_SIZE as _),
    staticvec_bench_extend => gen_extend(SPILLED_SIZE as _),
    staticvec_bench_extend_small => gen_extend(VEC_SIZE as _),
    staticvec_bench_from_iter => gen_from_iter(SPILLED_SIZE as _),
    staticvec_bench_from_iter_small => gen_from_iter(VEC_SIZE as _),
    staticvec_bench_from_slice => gen_from_slice(SPILLED_SIZE as _),
    staticvec_bench_from_slice_small => gen_from_slice(VEC_SIZE as _),
    staticvec_bench_extend_from_slice => gen_extend_from_slice(SPILLED_SIZE as _),
    staticvec_bench_extend_from_slice_small => gen_extend_from_slice(VEC_SIZE as _),
    staticvec_bench_macro_from_elem => gen_from_elem(SPILLED_SIZE as _),
    staticvec_bench_pushpop => gen_pushpop(),
  }
}

make_benches! {
  StaticVec<u64, {VEC_SIZE}> {
    staticvec_bench_macro_from_elem_small => gen_from_elem(VEC_SIZE as _),
  }
}

make_benches! {
  Vec<u64> {
    vec_bench_push => gen_push(SPILLED_SIZE as _),
    vec_bench_push_small => gen_push(VEC_SIZE as _),
    vec_bench_insert => gen_insert(SPILLED_SIZE_TWO as _),
    vec_bench_insert_small => gen_insert(VEC_SIZE as _),
    vec_bench_remove => gen_remove(SPILLED_SIZE as _),
    vec_bench_remove_small => gen_remove(VEC_SIZE as _),
    vec_bench_extend => gen_extend(SPILLED_SIZE as _),
    vec_bench_extend_small => gen_extend(VEC_SIZE as _),
    vec_bench_from_iter => gen_from_iter(SPILLED_SIZE as _),
    vec_bench_from_iter_small => gen_from_iter(VEC_SIZE as _),
    vec_bench_from_slice => gen_from_slice(SPILLED_SIZE as _),
    vec_bench_from_slice_small => gen_from_slice(VEC_SIZE as _),
    vec_bench_extend_from_slice => gen_extend_from_slice(SPILLED_SIZE as _),
    vec_bench_extend_from_slice_small => gen_extend_from_slice(VEC_SIZE as _),
    vec_bench_macro_from_elem => gen_from_elem(SPILLED_SIZE as _),
    vec_bench_macro_from_elem_small => gen_from_elem(VEC_SIZE as _),
    vec_bench_pushpop => gen_pushpop(),
  }
}

fn gen_push<V: Vector<u64>>(n: u64, b: &mut Bencher) {
  #[inline(never)]
  fn push_noinline<V: Vector<u64>>(vec: &mut V, x: u64) {
    vec.push(x);
  }

  b.iter(|| {
    let mut vec = V::new();
    for x in 0..n {
      push_noinline(&mut vec, x);
    }
    vec
  });
}

fn gen_insert<V: Vector<u64>>(n: u64, b: &mut Bencher) {
  #[inline(never)]
  fn insert_noinline<V: Vector<u64>>(vec: &mut V, p: usize, x: u64) {
    vec.insert(p, x)
  }

  b.iter(|| {
    let mut vec = V::new();
    // Add one element, with each iteration we insert one before the end.
    // This means that we benchmark the insertion operation and not the
    // time it takes to `ptr::copy` the data.
    vec.push(0);
    for x in 0..n {
      insert_noinline(&mut vec, x as _, x);
    }
    vec
  });
}

fn gen_remove<V: Vector<u64>>(n: usize, b: &mut Bencher) {
  #[inline(never)]
  fn remove_noinline<V: Vector<u64>>(vec: &mut V, p: usize) -> u64 {
    vec.remove(p)
  }

  b.iter(|| {
    let mut vec = V::from_elem(0, n as _);

    for x in (0..n - 1).rev() {
      remove_noinline(&mut vec, x);
    }
  });
}

fn gen_extend<V: Vector<u64>>(n: u64, b: &mut Bencher) {
  b.iter(|| {
    let mut vec = V::new();
    vec.extend(0..n);
    vec
  });
}

fn gen_from_iter<V: Vector<u64>>(n: u64, b: &mut Bencher) {
  let v: Vec<u64> = (0..n).collect();
  b.iter(|| {
    let vec = V::from(&v);
    vec
  });
}

fn gen_from_slice<V: Vector<u64>>(n: u64, b: &mut Bencher) {
  let v: Vec<u64> = (0..n).collect();
  b.iter(|| {
    let vec = V::from_elems(&v);
    vec
  });
}

fn gen_extend_from_slice<V: Vector<u64>>(n: u64, b: &mut Bencher) {
  let v: Vec<u64> = (0..n).collect();
  b.iter(|| {
    let mut vec = V::new();
    vec.extend_from_slice(&v);
    vec
  });
}

fn gen_pushpop<V: Vector<u64>>(b: &mut Bencher) {
  #[inline(never)]
  fn pushpop_noinline<V: Vector<u64>>(vec: &mut V, x: u64) -> Option<u64> {
    vec.push(x);
    vec.pop()
  }

  b.iter(|| {
    let mut vec = V::new();
    for x in 0..SPILLED_SIZE as _ {
      pushpop_noinline(&mut vec, x);
    }
    vec
  });
}

fn gen_from_elem<V: Vector<u64>>(n: usize, b: &mut Bencher) {
  b.iter(|| {
    let vec = V::from_elem(42, n);
    vec
  });
}

#[bench]
fn staticvec_bench_macro_from_list(b: &mut Bencher) {
  b.iter(|| {
    let vec = staticvec![
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 20, 24, 32, 36, 0x40, 0x80, 0x100,
      0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000,
      0x100000,
    ];
    vec
  });
}

#[bench]
fn vec_bench_macro_from_list(b: &mut Bencher) {
  b.iter(|| {
    let vec: Vec<u64> = vec![
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 20, 24, 32, 36, 0x40, 0x80, 0x100,
      0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000,
      0x100000,
    ];
    vec
  });
}

#[bench]
fn bench_clone(b: &mut Bencher) {
  let v: StaticVec<Vec<u32>, {200}> = (0..100)
    .map(|i| (0..i).collect())
    .collect();

  b.iter(move || {
    let _v2 = black_box(v.clone());
  });
}

#[bench]
fn bench_clone_from_shorter(b: &mut Bencher) {
  // We create some vectors with semi-random lengths to provoke
  // different behaviors in the underlying Vec::clone_from. This allows us to
  // demonstrate the advantage of using an underlying clone_from over a raw
  // clone.
  let src: StaticVec<Vec<u32>, {200}> = (0..50)
    .map(|i| (0..i % 7).collect())
    .collect();

  b.iter(move || {
    // TODO: find a way to make the bencher not include the time required to
    // instantiate `dst`. We can't move it outside of b.iter because we need
    // to make sure dst is in the same state before each bench run. We don't
    // want to clone it, either, because part of the test includes the
    // allocations into `vec`
    let mut dst: StaticVec<Vec<u32>, {200}> = black_box((0..100)
    .map(|i| {
      // ensure we have enouch capacity to benefit from clone_from
      let mut vec = Vec::with_capacity(20);
      vec.extend(1..1+(i % 11));
      vec
    })
    .collect());

    dst.clone_from(&src);
  });
}

#[bench]
fn bench_clone_from_longer(b: &mut Bencher) {
  // We create some vectors with semi-random lengths to provoke
  // different behaviors in the underlying Vec::clone_from. This allows us to
  // demonstrate the advantage of using an underlying clone_from over a raw
  // clone.
  let src: StaticVec<Vec<u32>, {200}> = (0..100)
    .map(|i| (0..i % 7).collect())
    .collect();

  b.iter(move || {
    // TODO: find a way to make the bencher not include the time required to
    // instantiate `dst`. We can't move it outside of b.iter because we need
    // to make sure dst is in the same state before each bench run. We don't
    // want to clone it, either, because part of the test includes the
    // allocations into `vec`
    let mut dst: StaticVec<Vec<u32>, {200}> = black_box((0..50)
    .map(|i| {
      // ensure we have enouch capacity to benefit from clone_from
      let mut vec = Vec::with_capacity(20);
      vec.extend(1..1+(i % 11));
      vec
    })
    .collect());

    dst.clone_from(&src);
  });
}
