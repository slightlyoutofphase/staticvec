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
fn staticvec_push_u32_255(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 255>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
    }
    v[4095]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u32_8192(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 8192>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(i);
    }
    v[8191]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u32_8192(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 8192]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(i);
    }
    v[8191]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u64_255(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 255>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
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
      v.push(i);
    }
    v[4095]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_push_u64_8192(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 8192>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(i);
    }
    v[8191]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn arrayvec_push_u64_8192(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 8192]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(i);
    }
    v[8191]
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn staticvec_pop_u32_255(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 255>::from([128; 255]);
    for i in 0..255 {
      v.pop.unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn arrayvec_pop_u32_255(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 255]>::from([128; 255]);
    for i in 0..255 {
      v.pop.unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn staticvec_pop_u32_512(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 512>::from([128; 512]);
    for i in 0..512 {
      v.pop.unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn arrayvec_pop_u32_512(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 512]>::from([128; 512]);
    for i in 0..512 {
      v.pop.unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn staticvec_pop_u32_1024(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 1024>::from([128; 1024]);
    for i in 0..1024 {
      v.pop.unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn arrayvec_pop_u32_1024(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 1024]>::from([128; 1024]);
    for i in 0..1024 {
      v.pop.unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn staticvec_pop_u32_2048(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 2048>::from([128; 2048]);
    for i in 0..2048 {
      v.pop.unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn arrayvec_pop_u32_2048(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 2048]>::from([128; 2048]);
    for i in 0..2048 {
      v.pop.unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn staticvec_pop_u32_4096(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 4096>::from([128; 4096]);
    for i in 0..4096 {
      v.pop.unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn arrayvec_pop_u32_4096(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 4096]>::from([128; 4096]);
    for i in 0..4096 {
      v.pop.unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn staticvec_pop_u32_8192(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 8192>::from([128; 8192]);
    for i in 0..8192 {
      v.pop.unwrap();
    }
  });
  b.bytes = 8192 as u64;
}

#[bench]
fn arrayvec_pop_u32_8192(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 8192]>::from([128; 8192]);
    for i in 0..8192 {
      v.pop.unwrap();
    }
  });
  b.bytes = 8192 as u64;
}

#[bench]
fn staticvec_pop_u64_255(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 255>::from([128; 255]);
    for i in 0..255 {
      v.pop.unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn arrayvec_pop_u64_255(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 255]>::from([128; 255]);
    for i in 0..255 {
      v.pop.unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn staticvec_pop_u64_512(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 512>::from([128; 512]);
    for i in 0..512 {
      v.pop.unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn arrayvec_pop_u64_512(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 512]>::from([128; 512]);
    for i in 0..512 {
      v.pop.unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn staticvec_pop_u64_1024(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 1024>::from([128; 1024]);
    for i in 0..1024 {
      v.pop.unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn arrayvec_pop_u64_1024(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 1024]>::from([128; 1024]);
    for i in 0..1024 {
      v.pop.unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn staticvec_pop_u64_2048(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 2048>::from([128; 2048]);
    for i in 0..2048 {
      v.pop.unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn arrayvec_pop_u64_2048(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 2048]>::from([128; 2048]);
    for i in 0..2048 {
      v.pop.unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn staticvec_pop_u64_4096(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 4096>::from([128; 4096]);
    for i in 0..4096 {
      v.pop.unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn arrayvec_pop_u64_4096(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 4096]>::from([128; 4096]);
    for i in 0..4096 {
      v.pop.unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn staticvec_pop_u64_8192(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 8192>::from([128; 8192]);
    for i in 0..8192 {
      v.pop.unwrap();
    }
  });
  b.bytes = 8192 as u64;
}

#[bench]
fn arrayvec_pop_u64_8192(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 8192]>::from([128; 8192]);
    for i in 0..8192 {
      v.pop.unwrap();
    }
  });
  b.bytes = 8192 as u64;
}
