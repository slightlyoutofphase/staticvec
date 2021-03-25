// So we don't get "function complexity" lints and such since it's a demo.
#![allow(clippy::all)]
// So we don't get warned about intentionally calling `drain_filter()` on a const struct,
// or warned about incomplete features.
#![allow(const_item_mutation, incomplete_features)]
#![feature(
  const_fn,
  const_fn_floating_point_arithmetic,
  const_maybe_uninit_assume_init,
  const_mut_refs,
  // Weirdly named feature, as it doesn't necessarily relate to `Cell` at all.
  const_refs_to_cell,
  const_trait_impl,
  inline_const,
  maybe_uninit_ref,
  type_name_of_val
)]

use core::mem::MaybeUninit;
use staticvec::{sortedstaticvec, staticvec, FromIterator, StaticVec};

#[derive(Copy, Clone, Debug)]
struct ZST {}

#[derive(Copy, Clone, Debug)]
struct MyStruct {
  s: &'static str,
}

impl MyStruct {
  pub const fn new(s: &'static str) -> Self {
    Self { s }
  }
}

#[derive(Debug)]
struct MyOtherStruct {
  s: &'static str,
}

impl Clone for MyOtherStruct {
  fn clone(&self) -> Self {
    Self { s: self.s }
  }
}

impl Drop for MyOtherStruct {
  fn drop(&mut self) {
    println!("Dropping MyOtherStruct with value: {}", self.s)
  }
}

// A little demonstration of some of the things you can do at compile time
static S1: StaticVec<usize, 6> = staticvec![1, 2, 3, 4, 5, 6];
static S2: StaticVec<usize, 4> = staticvec![1, 2, 3, 4];
static B: bool = S1.len() + S2.len() == 10;
static S3: StaticVec<bool, 2> = staticvec![B, !B];
static SLICE: &[bool] = S3.as_slice();
static LOL: StaticVec<usize, 12> = S1.intersperse(999);
static mut MUTABLE: StaticVec<usize, 128> = StaticVec::<usize, 128>::new();
static LEFT: StaticVec<MyStruct, 4> = staticvec![
  MyStruct::new("a"),
  MyStruct::new("b"),
  MyStruct::new("c"),
  MyStruct::new("d")
];
static RIGHT: StaticVec<MyStruct, 2> = staticvec![MyStruct::new("e"), MyStruct::new("f")];
static CONCATENATED: StaticVec<MyStruct, 6> = LEFT.concat(&RIGHT);

// It's also possible to write compile-time initialization functions that suit your specific needs
// with "complex" logic taking advantage of various methods you might typically expect to only
// be available at runtime.
const fn build<T: Copy, const N: usize>(x: [T; N]) -> StaticVec<T, N> {
  // StaticVec implements `Drop`, and so can't *directly* be returned from a `const fn` yet (at
  // least specifically if / when first instantiated as a fully-initialized "naked" function-local
  // variable) even when `T` is `Copy`, making the (sound) use of MaybeUninit below necessary to
  // facilitate regular access to the StaticVec we'll be creating.
  let mut mu = MaybeUninit::new(StaticVec::new());
  // Not actually unsafe here: `sv` is just a mutable reference to a normal empty StaticVec.
  let sv = unsafe { mu.assume_init_mut() };
  // `StaticVec::push` is a `const fn`. Note that loops in `const fn` are limited to `while`
  // currently: if that changes, obviously use an iterator-based `for` loop instead.
  let mut i = 0;
  while i < N {
    sv.push(x[i]);
    i += 1;
  }
  // `StaticVec::pop` is also a `const fn`, so we can do this as well...
  while sv.pop().is_some() {}
  // And put everything back in like this...
  sv.insert_from_slice(0, &x);
  // Still not actually unsafe here: we soundly initialized everything that needed it as soon as we
  // called `StaticVec::new()`.
  unsafe { mu.assume_init() }
  // The methods demonstrated above are by no means the only ones for StaticVec that could be used
  // (and might be useful) in a context like this, so do go ahead and peruse the docs for this crate
  // to find out more.
}

const BUILT: StaticVec<u8, 3> = build([2, 4, 6]);

fn main() {
  println!("{:?}", S1);
  println!("{:?}", S2);
  println!("{:?}", B);
  println!("{:?}", S3);
  println!("{:?}", SLICE);
  println!("{:?}", LOL);
  println!("{}", unsafe { MUTABLE.len() });
  println!("{}", unsafe { MUTABLE.capacity() });
  println!("{:?}", CONCATENATED);
  println!("{:?}", BUILT);
  let mut zz = StaticVec::<usize, 12>::from([1, 2, 3, 4, 5, 6]);
  let mut zzz = zz.clone();
  println!("{:?}", zzz);
  zzz.clear();
  zz.clone_from(&zzz);
  println!("{:?}", zz);
  let mut v = StaticVec::<f32, 24>::new();
  for i in 0..v.capacity() {
    v.push(i as f32);
  }
  for f in &v {
    println!("{}", f);
  }
  v.clear();
  v.insert(0, 47.6);
  v.insert(1, 48.6);
  v.insert(2, 49.6);
  v.insert(v.len() - 1, 50.6);
  v.insert(v.len() - 2, 51.6);
  v.insert(v.len() - 3, 52.6);
  for f in &v {
    println!("{}", f);
  }
  for f in 0..v.len() {
    println!("{}", v[f]);
  }
  v.remove(1);
  v.remove(2);
  for f in &v {
    println!("{}", f);
  }
  let mut va = StaticVec::<usize, 128>::new();
  for i in 0..va.capacity() {
    va.push(i);
  }
  let ia = va.remove_item(&64).unwrap();
  let ib = va.remove_item(&63).unwrap();
  println!("{}", ia);
  println!("{}", ib);
  va.remove(10);
  va.remove(11);
  va.remove(12);
  va.remove(13);
  va.remove(14);
  va.remove(15);
  va.insert(10, 99);
  va.insert(11, 99);
  va.insert(12, 99);
  va.insert(13, 99);
  va.insert(14, 99);
  va.insert(15, 99);
  for i in 0..va.len() {
    println!("{}", va[i])
  }
  for i in &va {
    println!("{}", i)
  }
  while va.is_not_empty() {
    println!("{}", va.pop().unwrap());
  }
  let mut vb = StaticVec::<char, 26>::new();
  vb.push('a');
  vb.push('b');
  vb.push('c');
  vb.push('d');
  vb.push('e');
  vb.push('f');
  vb.push('g');
  vb.push('h');
  vb.push('i');
  vb.push('j');
  vb.push('k');
  vb.push('l');
  vb.push('m');
  vb.push('n');
  vb.push('o');
  vb.push('p');
  vb.push('q');
  vb.push('r');
  vb.push('s');
  vb.push('t');
  vb.push('u');
  vb.push('v');
  vb.push('w');
  vb.push('x');
  vb.push('y');
  vb.push('z');
  vb.remove(2);
  vb.remove(1);
  vb.remove(vb.len() - 1);
  for i in 0..vb.len() {
    println!("{}", vb[i]);
  }
  for s in &vb {
    println!("{}", s);
  }
  let pb = vb.as_mut_ptr();
  unsafe {
    println!("{}", *pb);
    println!("{}", *pb.add(1).add(1));
  }
  let pbc = vb.as_ptr();
  unsafe {
    println!("{}", *pbc);
    println!("{}", *pbc.add(1).add(1));
  }
  vb.clear();
  for _i in 0..vb.capacity() {
    vb.push('h');
  }
  while vb.is_not_empty() {
    println!("{}", vb.remove(0));
  }
  vb.push('g');
  vb.push('f');
  vb.push('e');
  vb.push('d');
  vb.push('c');
  vb.push('b');
  vb.push('a');
  let vbm = vb.as_mut_slice();
  vbm.sort_unstable();
  for s in vbm {
    println!("{}", s);
  }
  let vbmb = vb.as_mut_slice();
  vbmb.reverse();
  for s in vbmb {
    println!("{}", s);
  }
  for s in &vb.sorted_unstable() {
    println!("{}", s);
  }
  for s in &vb.reversed() {
    println!("{}", s);
  }
  vb.reverse();
  vb.reverse();
  for s in &vb {
    println!("{}", s);
  }
  vb.clear();
  let mut vu = StaticVec::<usize, 8>::new();
  vu.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
  println!("{}", vu.drain(2..5).iter().find(|&&i| i == 4).unwrap());
  let vvu: StaticVec<&usize, 4> = vu.iter().collect();
  for i in &vvu {
    println!("{}", i);
  }
  let mut x = StaticVec::<i32, 4>::new();
  x.push(1);
  x.push(2);
  x.push(3);
  x.push(4);
  let mut y = StaticVec::<i32, 4>::new();
  y.push(4);
  y.push(3);
  y.push(2);
  y.push(1);
  let mut z = StaticVec::<i32, 4>::new();
  z.push(1);
  z.push(2);
  z.push(3);
  z.push(4);
  let mut w = StaticVec::<i32, 4>::new();
  w.push(4);
  w.push(3);
  w.push(2);
  w.push(1);
  let mut ww = StaticVec::<&StaticVec<i32, 4>, 4>::new();
  ww.push(&x);
  ww.push(&y);
  ww.push(&z);
  ww.push(&w);
  for v in &ww {
    for i in *v {
      println!("{}", i);
    }
  }
  let mut empty = StaticVec::<&'static str, 0>::new();
  empty.sort_unstable();
  empty.reverse();
  for s in empty.as_slice() {
    println!("{}", s);
  }
  for s in empty.as_mut_slice() {
    println!("{}", s);
  }
  for s in &empty {
    println!("{}", s);
  }
  for s in &mut empty {
    println!("{}", s);
  }
  for s in &empty.reversed() {
    println!("{}", s);
  }
  for s in &empty.sorted_unstable() {
    println!("{}", s);
  }
  let mut msv = StaticVec::<MyStruct, 4>::new();
  msv.push(MyStruct { s: "a" });
  msv.push(MyStruct { s: "b" });
  msv.push(MyStruct { s: "c" });
  msv.push(MyStruct { s: "d" });
  for ms in &msv.reversed() {
    println!("{}", ms.s);
  }
  while msv.is_not_empty() {
    println!("{}", msv.remove(msv.len() - 1).s)
  }
  let v2 = StaticVec::<i32, 8>::new_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
  let mut it2 = v2.iter();
  println!("{:?}", it2.size_hint());
  while let Some(_i) = it2.next() {
    println!("{:?}", it2.size_hint());
    println!("{:?}", it2.len());
  }
  if let Some(i) = v2.iter().rfind(|&&x| x == 2) {
    println!("{}", i);
  }
  for v in &staticvec![
    staticvec![12.0, 14.0],
    staticvec![16.0, 18.0],
    staticvec![20.0, 22.0],
    staticvec![24.0, 26.0],
    staticvec![28.0, 30.0],
    staticvec![32.0, 34.0],
    staticvec![36.0, 38.0],
    staticvec![40.0, 42.0]
  ] {
    for f in v.iter().skip(1) {
      println!("{}", f);
    }
  }
  let numbers = staticvec![1, 2, 3, 4, 5];
  let zero = "0".to_string();
  let result = numbers
    .iter()
    .rfold(zero, |acc, &x| format!("({} + {})", x, acc));
  println!("{}", result);
  let mut strings = staticvec!["foo", "bar", "baz", "qux"];
  println!("{}", strings.swap_remove(1));
  for s in &strings {
    println!("{}", s);
  }
  println!("{}", strings.swap_remove(0));
  for s in &strings {
    println!("{}", s);
  }
  const STRUCTS: StaticVec<MyOtherStruct, 6> = staticvec![
    MyOtherStruct { s: "a" },
    MyOtherStruct { s: "b" },
    MyOtherStruct { s: "c" },
    MyOtherStruct { s: "d" },
    MyOtherStruct { s: "e" },
    MyOtherStruct { s: "f" },
  ];
  let mut newstructs = STRUCTS.drain_filter(|s| s.s < "d");
  for s in &STRUCTS {
    println!("{}", s.s);
  }
  for s in &newstructs {
    println!("{}", s.s);
  }
  newstructs.retain(|s| s.s == "a");
  for s in &newstructs {
    println!("{}", s.s);
  }
  let mut morestructs = staticvec![
    MyOtherStruct { s: "a" },
    MyOtherStruct { s: "b" },
    MyOtherStruct { s: "c" },
    MyOtherStruct { s: "d" },
    MyOtherStruct { s: "e" },
    MyOtherStruct { s: "f" },
  ];
  morestructs.truncate(3);
  for s in &morestructs {
    println!("{}", s.s);
  }
  let mut twelve = StaticVec::<f32, 12>::new();
  twelve.extend([1.0, 2.0, 3.0, 4.0, 5.0, 6.0].iter());
  for f in &twelve {
    println!("{}", f);
  }
  println!("{}", twelve.capacity());
  println!("{}", twelve.len());
  let single_element_macro = staticvec!["ABCD"; 26];
  for s in &single_element_macro {
    println!("{}", s);
  }
  let eight_from = StaticVec::<usize, 8>::from(staticvec![1, 2, 3, 4, 5, 6, 7, 8].as_slice());
  for i in &eight_from {
    println!("{}", i);
  }
  let twenty_from_ten = StaticVec::<i32, 20>::from([123; 10]);
  for i in &twenty_from_ten {
    println!("{}", i);
  }
  for i in &staticvec![1, 2, 3, 4, 5, 6, 7, 8].split_off(3) {
    println!("{}", i);
  }
  let mut strings2 = staticvec!["foo", "bar", "Bar", "baz", "bar"];
  strings2.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
  for s in &strings2 {
    println!("{}", s);
  }
  let mut ints = staticvec![10, 20, 21, 30, 20];
  ints.dedup_by_key(|i| *i / 10);
  for i in &ints {
    println!("{}", i);
  }
  let mut ints2 = staticvec![1, 2, 2, 3, 2];
  ints2.dedup();
  for i in &ints2 {
    println!("{}", i);
  }
  let mut y = StaticVec::<MyStruct, 2>::new();
  y.push(MyStruct { s: "hey" });
  for x in y.as_mut() {
    println!("{}", x.s);
  }
  println!("Size of ZST: {}", core::mem::size_of::<ZST>());
  let zsts = staticvec![ZST {}, ZST {}, ZST {}, ZST {}];
  for zst in &zsts {
    println!("{:?}", zst);
  }
  for zst in &zsts.reversed() {
    println!("{:?}", zst);
  }
  println!("{:?}", zsts.iter().size_hint());
  println!("{}", zsts.iter().len());
  let v = staticvec![
    Box::new(MyOtherStruct { s: "AAA" }),
    Box::new(MyOtherStruct { s: "BBB" }),
    Box::new(MyOtherStruct { s: "CCC" })
  ];
  println!("{} {}", v.capacity(), v.len());
  let vv = v.into_vec();
  println!("{} {}", vv.capacity(), vv.len());
  for s in vv {
    println!("{:?}", s);
  }
  let cc = staticvec!["AAA", "BBB"];
  let dd = cc.clone();
  let mut ee = StaticVec::new();
  ee.clone_from(&dd);
  for s in &ee.reversed() {
    println!("{}", s);
  }
  let clonee = staticvec![
    Box::new(MyOtherStruct { s: "AAA" }),
    Box::new(MyOtherStruct { s: "BBB" }),
    Box::new(MyOtherStruct { s: "CCC" }),
    Box::new(MyOtherStruct { s: "DDD" }),
    Box::new(MyOtherStruct { s: "EEE" }),
  ];
  let cloned = clonee.clone();
  for ms in &cloned {
    println!("{:?}", ms);
  }
  let v2 = StaticVec::from([
    Box::new(MyOtherStruct { s: "XYZ" }),
    Box::new(MyOtherStruct { s: "ZYX" }),
    Box::new(MyOtherStruct { s: "XYZ" }),
  ]);
  let mut it = v2.into_iter();
  println!("{:?}", it.next().unwrap());
  println!("{:?}", it.next().unwrap());
  let mut to_append = staticvec![1, 2, 3];
  let mut append_to = StaticVec::<i32, 6>::from([4, 5, 6]);
  append_to.append(&mut to_append);
  println!("{:?}", to_append);
  println!("{:?}", append_to);
  static STATIC_STATICVEC: StaticVec<u8, 5> = staticvec![1, 2, 3, 4, 5];
  println!(
    "{:?}",
    STATIC_STATICVEC
      .intersperse(7)
      .reversed()
      .sorted_unstable()
      .drain_iter(4..7)
  );
  let mut extended = StaticVec::<u8, 12>::new();
  extended.extend(staticvec![1, 2, 3].iter());
  extended.extend(staticvec![4, 5, 6].into_iter());
  let yyz = StaticVec::<u8, 6>::from_iter(staticvec![4, 5, 6].iter());
  println!("{:?}", yyz);
  let zzy = StaticVec::<u8, 6>::from_iter(staticvec![4, 5, 6].iter().copied());
  println!("{:?}", zzy);
  let zwz = StaticVec::<u8, 3>::from_iter(staticvec![4, 5, 6].iter().cloned());
  println!("{:?}", zwz);
  static V: StaticVec<f64, 3> = sortedstaticvec!(f64, [16.0, 15.0, 14.0]);
  println!("{:?}", V);
  println!("{:?}", V.reversed().drain(0..1));
  static VV: StaticVec<f64, 0> = sortedstaticvec!(f64, []);
  println!("{:?}", VV);
  // The type parameter is inferred as `StaticVec<usize, 8>`.
  let filled = StaticVec::<_, 128>::filled_with_by_index(|i| {
    staticvec![i + 1, i + 2, i + 3, i + 4,].intersperse((i + 4) * 4)
  });
  println!("{:?}", filled);
  println!("{}", std::any::type_name_of_val(&filled));
  println!("{}", filled[0].remaining_capacity());
}
