#![allow(incomplete_features)]
#![allow(clippy::all)]
#![feature(test)]
#![feature(const_generics)]

extern crate test;

use std::io::Write;
use test::{black_box, Bencher};

use arrayvec::*;
use staticvec::*;

#[bench]
fn staticvec_extend_from_slice(b: &mut Bencher) {
  let mut v = StaticVec::<u8, 512>::new();
  let data = [1; 512];
  b.iter(|| {
    v.clear();
    black_box(v.try_extend_from_slice(&data).ok());
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
    black_box(v.try_extend_from_slice(&data).ok());
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

#[bench]
fn staticvec_push_u32_255(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 255>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      black_box(v.push(i));
    }
    v[254]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u32_255(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 255]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      black_box(v.push(i));
    }
    v[254]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u32_512(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 512>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      black_box(v.push(i));
    }
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u32_512(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 512]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      black_box(v.push(i));
    }
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u32_1024(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 1024>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      black_box(v.push(i));
    }
    v[1023]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u32_1024(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 1024]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      black_box(v.push(i));
    }
    v[1023]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u32_2048(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 2048>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      black_box(v.push(i));
    }
    v[2047]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u32_2048(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 2048]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      black_box(v.push(i));
    }
    v[2047]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u32_4096(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 4096>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      black_box(v.push(i));
    }
    v[4095]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u32_4096(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 4096]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      black_box(v.push(i));
    }
    v[4095]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u64_255(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 255>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      black_box(v.push(i));
    }
    v[254]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u64_255(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 255]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      black_box(v.push(i));
    }
    v[254]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u64_512(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 512>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      black_box(v.push(i));
    }
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u64_512(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 512]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      black_box(v.push(i));
    }
    v[511]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u64_1024(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 1024>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      black_box(v.push(i));
    }
    v[1023]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u64_1024(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 1024]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      black_box(v.push(i));
    }
    v[1023]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u64_2048(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 2048>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      black_box(v.push(i));
    }
    v[2047]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u64_2048(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 2048]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      black_box(v.push(i));
    }
    v[2047]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u64_4096(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 4096>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      black_box(v.push(i));
    }
    v[4095]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u64_4096(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 4096]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      black_box(v.push(i));
    }
    v[4095]
  });
  b.bytes = v.capacity() as u64;
}
