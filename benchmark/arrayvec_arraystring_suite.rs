#![feature(test)]

extern crate test;

use test::Bencher;

use staticvec::StaticString;

#[bench]
fn try_push_c(b: &mut Bencher) {
  let mut v = StaticString::<512>::new();
  b.iter(|| {
    v.clear();
    while v.try_push('c').is_ok() {}
    v.len()
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn try_push_alpha(b: &mut Bencher) {
  let mut v = StaticString::<512>::new();
  b.iter(|| {
    v.clear();
    while v.try_push('α').is_ok() {}
    v.len()
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn try_push_string(b: &mut Bencher) {
  let mut v = StaticString::<512>::new();
  let input = "abcαβγ“”";
  b.iter(|| {
    v.clear();
    for ch in input.chars().cycle() {
      if !v.try_push(ch).is_ok() {
        break;
      }
    }
    v.len()
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn push_c(b: &mut Bencher) {
  let mut v = StaticString::<512>::new();
  b.iter(|| {
    v.clear();
    while !v.is_full() {
      v.push('c');
    }
    v.len()
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn push_alpha(b: &mut Bencher) {
  let mut v = StaticString::<512>::new();
  b.iter(|| {
    v.clear();
    while !v.is_full() {
      v.push('α');
    }
    v.len()
  });
  b.bytes = v.capacity() as u64;
}

#[bench]
fn push_string(b: &mut Bencher) {
  let mut v = StaticString::<512>::new();
  let input = "abcαβγ“”";
  b.iter(|| {
    v.clear();
    for ch in input.chars().cycle() {
      if !v.is_full() {
        v.push(ch);
      } else {
        break;
      }
    }
    v.len()
  });
  b.bytes = v.capacity() as u64;
}
