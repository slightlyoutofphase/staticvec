**Note:** I am aware that changes were made on around June 4th that reverted the constness of various pointer methods. As these have been explicitly stated as temporary in the relevant PRs, no "fix version" of this crate will be issued unless the changes in question prove to be stretching the truth about how temporary they actually are.

[![Latest Version]][crates.io] ![Rustc Version nightly]

[Latest Version]: https://img.shields.io/crates/v/staticvec.svg
[crates.io]: https://crates.io/crates/staticvec
[Rustc Version nightly]: https://img.shields.io/badge/rustc-nightly-lightgray.svg
[![Build status](https://ci.appveyor.com/api/projects/status/qb40my4v3rr63st2/branch/master?svg=true)](https://ci.appveyor.com/project/slightlyoutofphase/staticvec/branch/master)

Implements a fixed-capacity stack-allocated `Vec` alternative backed by an array, using const generics.

Note: the word "static" here is meant by the traditional definition of "unchanging" or "not dynamic".

This crate does **not** use literal `static` variables for anything (but does provide multiple ways
to instantiate a `StaticVec` **as** a `static` or `const` variable if desired).

Fully `#![no_std]` compatible (with almost no loss of functionality) by setting
`default-features = false` for the `staticvec` dependency in your `Cargo.toml`.

Optional support for serialization and deserialization of the `StaticVec` struct
via `serde` is available by activating the `serde_support` crate feature.

`StaticVec` also implements both `Deref` and `DerefMut` to `[T]`, meaning that all existing slice
methods are accessible through instances of it and that references to it can be used in contexts
where `[T]` is expected.

As of version 0.8.0, this crate additionally provides a fixed-capacity `StaticString` struct, which
is built around an instance of `StaticVec<u8, N>`.

As of version 0.8.5, a fixed-capacity `StaticHeap` struct based on the standard library `BinaryHeap`
and built around an instance of `StaticVec<T, N>` has been added as well.

Contributions/suggestions/etc. very welcome!

**Minimum supported Rust version:** this is a nightly-only crate at the moment due to the use of
various feature gates that provide functionality beyond the scope of stabilized minimal subsets
such as `min_const_generics` and `min_const_fn`.

A basic usage example:

```rust
use staticvec::{staticvec, StaticVec};

fn main() {
  let mut v = StaticVec::<usize, 64>::new();
  for i in 0..v.capacity() {
    v.push(i);
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
  for i in staticvec![
    staticvec![14, 12, 10].sorted(),
    staticvec![20, 18, 16].reversed(),
    staticvec![26, 24, 22].sorted(),
    staticvec![32, 30, 28].reversed(),
  ]
  .iter()
  .flatten()
  .collect::<StaticVec<usize, 12>>()
  .iter() {
    println!("{}", i);
  }
  // The type parameter is inferred as `StaticVec<usize, 16>`.
  let filled = StaticVec::<_, 6>::filled_with_by_index(|i| {
    staticvec![
      i + 1,
      i + 2,
      i + 3,
      i + 4,
    ]
    .concat(&staticvec![6, 6, 7, 7])
    .intersperse((i + 4) * 4)
  });
  println!("{:?}", filled);
}
```

**License:**

Licensed under either the <a href="LICENSE-MIT">MIT license</a> or version 2.0 of the <a href="LICENSE-APACHE">Apache License</a>. Your choice as to which!
Any source code contributions will be dual-licensed in the same fashion.
