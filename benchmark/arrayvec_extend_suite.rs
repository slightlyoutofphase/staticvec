#![allow(incomplete_features)]
#![allow(clippy::all)]
#![feature(test)]
#![feature(const_generics)]

extern crate test;

use std::io::Write;
use test::{black_box, Bencher};

use arrayvec::ArrayVec;
use staticvec::StaticVec;

#[bench]
fn staticvec_extend_from_slice(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    v.try_extend_from_slice(black_box(&data[..]));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_extend_from_slice(b: &mut Bencher) {
  let mut v = ArrayVec::<[u8; 512]>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    v.try_extend_from_slice(black_box(&data[..]));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_constant(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let cap = v.capacity();
  b.iter(|| {
    v.clear();
    let constant = 1;
    black_box(v.extend((0..cap).map(move |_| constant)));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_extend_with_constant(b: &mut Bencher) {
  let mut v = ArrayVec::<[u8; 512]>::new();
  let cap = v.capacity();
  b.iter(|| {
    v.clear();
    let constant = 1;
    black_box(v.extend((0..cap).map(move |_| constant)));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_range(b: &mut Bencher) {
  let mut v = StaticVec::<u16, 512>::new();
  let cap = v.capacity();
  b.iter(|| {
    v.clear();
    let range = 0..cap;
    black_box(v.extend(range.map(|x| x as u16)));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_extend_with_range(b: &mut Bencher) {
  let mut v = ArrayVec::<[u16; 512]>::new();
  let cap = v.capacity();
  b.iter(|| {
    v.clear();
    let range = 0..cap;
    black_box(v.extend(range.map(|x| x as u16)));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_slice(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    let iter = data.iter().map(|&x| x);
    black_box(v.extend(iter));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_extend_with_slice(b: &mut Bencher) {
  let mut v = ArrayVec::<[u8; 512]>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    let iter = data.iter().map(|&x| x);
    black_box(v.extend(iter));
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_extend_with_write(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    black_box(v.write(&data[..]).ok());
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_extend_with_write(b: &mut Bencher) {
  let mut v = ArrayVec::<[u8; 512]>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    black_box(v.write(&data[..]).ok());
    v[511]
  });
  b.bytes = v.capacity() as u64;
}
