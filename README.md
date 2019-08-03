[![Latest Version]][crates.io] [![Rustc Version nightly]][rustc]

[Latest Version]: https://img.shields.io/crates/v/staticvec.svg
[Rustc Version nightly]: https://img.shields.io/badge/rustc-nightly-lightgray.svg

Implements a fixed-capacity stack-allocated Vec alternative backed by a static array, using const generics.

Fully `#![no_std]` compatible (with almost no loss of functionality) by setting 
`default-features = false` for the `staticvec` dependency in your `Cargo.toml`.

Contributions/suggestions/e.t.c very welcome!

**Minimum supported Rust version:** due to the use of const generics, this is a nightly-only crate at the moment.

A basic usage example:

```rust
use staticvec::*;

fn main() {
  let mut v = StaticVec::<i32, 64>::new();
  for i in 0..v.capacity() {
    v.push(i as i32);
  }
  for i in &v {
    println!("{}", i);
  }
  v.clear();
  v.insert(0, 47);
  v.insert(1, 48);
  v.insert(2, 49);
  v.insert(v.len() - 1, 50);
  v.insert(v.len() - 2, 51);
  v.insert(v.len() - 3, 52);
  for i in &v {
    println!("{}", i);
  }
  for i in &v.reversed().drain(2..4) {
    println!("{}", i);
  }
  while v.is_not_empty() {
    println!("{}", v.remove(0));
  }
  for f in staticvec![12.0, 14.0, 15.0, 16.0].iter().skip(2) {
    println!("{}", f);
  }
  for v in &staticvec![
    staticvec![12.0, 14.0],
    staticvec![16.0, 18.0],
    staticvec![20.0, 22.0],
    staticvec![24.0, 26.0]
  ] {
    for f in v.iter().skip(1) {
      println!("{}", f);
    }
  }
}
```