#![allow(incomplete_features)]
#![allow(clippy::all)]
#![feature(test)]
#![feature(const_generics)]

extern crate test;

use test::{black_box, Bencher};

use staticvec::StaticVec;

#[bench]
fn staticvec_of_vecs_bench_clone(b: &mut Bencher) {
  let v: StaticVec<Vec<u32>, 200> = (0..100).map(|i| (0..i).collect()).collect();

  b.iter(move || {
    let _v2 = black_box(v.clone());
  });
}

#[bench]
fn staticvec_of_u32_bench_trivial_clone(b: &mut Bencher) {
  let v: StaticVec<u32, 200> = (50..150).collect();

  b.iter(move || {
    let _v2 = black_box(v.clone());
  })
}

#[bench]
fn staticvec_of_vecs_bench_clone_from_shorter(b: &mut Bencher) {
  // We create some vectors with semi-random lengths to provoke
  // different behaviors in the underlying Vec::clone_from. This allows us to
  // demonstrate the advantage of using an underlying clone_from over a raw
  // clone.
  let src: StaticVec<Vec<u32>, 200> = (0..50).map(|i| (0..i % 7).collect()).collect();

  b.iter(move || {
    // TODO: find a way to make the bencher not include the time required to
    // instantiate `dst`. We can't move it outside of b.iter because we need
    // to make sure dst is in the same state before each bench run. We don't
    // want to clone it, either, because part of the test includes the
    // allocations into `vec`
    let mut dst: StaticVec<Vec<u32>, 200> = (0..100)
      .map(|i| {
        // ensure we have enough capacity to benefit from clone_from
        let mut vec = Vec::with_capacity(20);
        vec.extend(1..1 + (i % 11));
        vec
      })
      .collect();
    black_box(dst.clone_from(&src));
  });
}

#[bench]
fn staticvec_of_vecs_bench_clone_from_longer(b: &mut Bencher) {
  // We create some vectors with semi-random lengths to provoke
  // different behaviors in the underlying Vec::clone_from. This allows us to
  // demonstrate the advantage of using an underlying clone_from over a raw
  // clone.
  let src: StaticVec<Vec<u32>, 200> = (0..100).map(|i| (0..i % 7).collect()).collect();

  b.iter(move || {
    // TODO: find a way to make the bencher not include the time required to
    // instantiate `dst`. We can't move it outside of b.iter because we need
    // to make sure dst is in the same state before each bench run. We don't
    // want to clone it, either, because part of the test includes the
    // allocations into `vec`
    let mut dst: StaticVec<Vec<u32>, 200> = (0..50)
      .map(|i| {
        // ensure we have enouch capacity to benefit from clone_from
        let mut vec = Vec::with_capacity(20);
        vec.extend(1..1 + (i % 11));
        vec
      })
      .collect();

    black_box(dst.clone_from(&src));
  });
}
