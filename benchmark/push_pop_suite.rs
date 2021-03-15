#![allow(clippy::all, incomplete_features)]
#![feature(const_generics, test)]

extern crate test;

use test::{black_box, Bencher};

use staticvec::{staticvec, StaticVec};

#[bench]
fn staticvec_push_u32_255_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u32_512_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u32_1024_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u32_2048_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u32_4096_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u32_8192_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u64_255_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u64_512_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u64_1024_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u64_2048_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u64_4096_blackboxed(b: &mut Bencher) {
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
fn staticvec_push_u64_8192_blackboxed(b: &mut Bencher) {
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
fn staticvec_pop_u32_255_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u32; 255];
    for _ in 0..255 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn staticvec_pop_u32_512_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u32; 512];
    for _ in 0..512 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn staticvec_pop_u32_1024_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u32; 1024];
    for _ in 0..1024 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn staticvec_pop_u32_2048_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u32; 2048];
    for _ in 0..2048 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn staticvec_pop_u32_4096_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u32; 4096];
    for _ in 0..4096 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn staticvec_pop_u32_8192_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u32; 8192];
    for _ in 0..8192 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 8192 as u64;
}

#[bench]
fn staticvec_pop_u64_255_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u64; 255];
    for _ in 0..255 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 255 as u64;
}

#[bench]
fn staticvec_pop_u64_512_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u64; 512];
    for _ in 0..512 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 512 as u64;
}

#[bench]
fn staticvec_pop_u64_1024_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u64; 1024];
    for _ in 0..1024 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 1024 as u64;
}

#[bench]
fn staticvec_pop_u64_2048_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u64; 2048];
    for _ in 0..2048 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 2048 as u64;
}

#[bench]
fn staticvec_pop_u64_4096_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u64; 4096];
    for _ in 0..4096 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 4096 as u64;
}

#[bench]
fn staticvec_pop_u64_8192_blackboxed(b: &mut Bencher) {
  b.iter(|| {
    let mut v = staticvec![128u64; 8192];
    for _ in 0..8192 {
      black_box(v.pop().unwrap());
    }
  });
  b.bytes = 8192 as u64;
}
