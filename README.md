Implements a fixed-capacity Vec alternative backed by a static array using const generics.

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
    println!("{}", f);
  }
  for i in 0..v.len() {
    println!("{}", v[i]);
  }
  v.remove(1);
  v.remove(2);
  for i in &v {
    println!("{}", i);
  }
}
```