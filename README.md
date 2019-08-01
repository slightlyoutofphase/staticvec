Implements a fixed-capacity Vec alternative backed by a static array using const generics.

**Minimum supported Rust version:** due to the use of const generics, this is a nightly-only crate at the moment.

A basic usage example:

```rust
use staticvec::{staticvec, StaticVec};

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
}
```