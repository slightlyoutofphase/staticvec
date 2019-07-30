Implements a fixed-capacity Vec alternative backed by a static array using const generics.

**Minimum supported Rust version:** due to the use of const generics, this is a nightly-only crate at the moment.

A basic usage example:

```rust
use staticvec::StaticVec;

fn main() {
  let mut v = StaticVec::<i32, 24>::new();
  for _i in 0..v.capacity() {
    v.push(42);
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
  for i in &v.reversed() {
    println!("{}", i);
  }
  while v.is_not_empty() {
    println!("{}", v.remove(0));
  }
}
```