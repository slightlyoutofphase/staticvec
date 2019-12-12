#![allow(incomplete_features)]
#![allow(clippy::all)]
#![feature(test)]
#![feature(const_generics)]

extern crate test;

use test::{black_box, Bencher};

use arrayvec::ArrayVec;
use staticvec::StaticVec;

#[bench]
fn u32_255_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 255>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_255_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 255]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_512_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 512>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_512_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 512]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_1024_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 1024>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_1024_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 1024]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_2048_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 2048>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_2048_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 2048]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_4096_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 4096>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_4096_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 4096]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_8192_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u32, 8192>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_8192_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u32; 8192]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_255_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 255>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_255_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 255]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..255 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_512_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 512>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_512_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 512]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..512 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_1024_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 1024>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_1024_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 1024]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..1024 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_2048_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 2048>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_2048_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 2048]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..2048 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_4096_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 4096>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_4096_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 4096]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..4096 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_8192_staticvec_push(b: &mut Bencher) {
  let mut v = StaticVec::<u64, 8192>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u64_8192_arrayvec_push(b: &mut Bencher) {
  let mut v = ArrayVec::<[u64; 8192]>::new();
  b.iter(|| {
    v.clear();
    for i in 0..8192 {
      v.push(black_box(i));
    }
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn u32_255_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 255>::from([128; 255]);
    for _ in 0..255 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn u32_255_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 255]>::from([128; 255]);
    for _ in 0..255 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn u32_512_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 512>::from([128; 512]);
    for _ in 0..512 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn u32_512_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 512]>::from([128; 512]);
    for _ in 0..512 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn u32_1024_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 1024>::from([128; 1024]);
    for _ in 0..1024 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn u32_1024_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 1024]>::from([128; 1024]);
    for _ in 0..1024 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn u32_2048_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 2048>::from([128; 2048]);
    for _ in 0..2048 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn u32_2048_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 2048]>::from([128; 2048]);
    for _ in 0..2048 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn u32_4096_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 4096>::from([128; 4096]);
    for _ in 0..4096 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn u32_4096_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 4096]>::from([128; 4096]);
    for _ in 0..4096 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn u32_8192_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u32, 8192>::from([128; 8192]);
    for _ in 0..8192 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 8192 as u64;
}

#[bench]
fn u32_8192_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u32; 8192]>::from([128; 8192]);
    for _ in 0..8192 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 8192 as u64;
}

#[bench]
fn u64_255_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 255>::from([128; 255]);
    for _ in 0..255 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn u64_255_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 255]>::from([128; 255]);
    for _ in 0..255 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn u64_512_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 512>::from([128; 512]);
    for _ in 0..512 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn u64_512_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 512]>::from([128; 512]);
    for _ in 0..512 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn u64_1024_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 1024>::from([128; 1024]);
    for _ in 0..1024 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn u64_1024_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 1024]>::from([128; 1024]);
    for _ in 0..1024 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn u64_2048_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 2048>::from([128; 2048]);
    for _ in 0..2048 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn u64_2048_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 2048]>::from([128; 2048]);
    for _ in 0..2048 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn u64_4096_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 4096>::from([128; 4096]);
    for _ in 0..4096 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn u64_4096_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 4096]>::from([128; 4096]);
    for _ in 0..4096 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn u64_8192_staticvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = StaticVec::<u64, 8192>::from([128; 8192]);
    for _ in 0..8192 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 8192 as u64;
}

#[bench]
fn u64_8192_arrayvec_pop(b: &mut Bencher) {
  b.iter(|| {
    let mut v = ArrayVec::<[u64; 8192]>::from([128; 8192]);
    for _ in 0..8192 {
      black_box(v.pop()).unwrap();
    }
  });
  b.bytes = 8192 as u64;
}
