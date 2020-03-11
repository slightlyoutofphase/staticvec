#![allow(incomplete_features)]
#![allow(clippy::all)]
#![feature(test)]
#![feature(const_generics)]

extern crate test;

use test::{black_box, Bencher};

use std::io::Write;

use staticvec::StaticVec;

#[bench]
fn staticvec_extend_from_slice_blackboxed(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    v.try_extend_from_slice(black_box(&data[..])).ok();
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_constant_blackboxed(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let cap = v.capacity();
  b.iter(|| {
    v.clear();
    let constant = black_box(1);
    v.extend((0..cap).map(move |_| constant));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_range_blackboxed(b: &mut Bencher) {
  let mut v = StaticVec::<u16, 512>::new();
  let cap = v.capacity();
  b.iter(|| {
    v.clear();
    let range = 0..cap;
    v.extend(black_box(range).map(|x| x as u16));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_slice_blackboxed(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    let iter = data.iter().map(|&x| x);
    // The black box kind of makes this one *too* slow I think,
    // but without it (as is also the case for several of the other bench functions)
    // it always runs in 0 ns.
    black_box(v.extend(iter));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_write_blackboxed(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    v.write(&data[..]).ok();
    black_box(v[511])
  });
  b.bytes = v.capacity() as u64;
}
