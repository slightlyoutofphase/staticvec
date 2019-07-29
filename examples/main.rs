use staticvec::StaticVec;
use std::iter::FromIterator;

fn bloop() {
  let mut x = Vec::<&i32>::with_capacity(4);
  x.push(&1);
  x.push(&2);
  x.push(&3);
  x.push(&4);
  let mut y = Vec::<&i32>::with_capacity(4);;
  y.push(&4);
  y.push(&3);
  y.push(&2);
  y.push(&1);
  let mut z = StaticVec::<&Vec<&i32>, 2>::new();
  z.push(&x);
  z.push(&y);
  for v in &z {
    for i in *v {
      println!("{}", i);
    }
  }
}

struct MyStruct {
  s: &'static str,
}

impl Drop for MyStruct {
  fn drop(&mut self) {
    println!("{}", "dropping");
  }
}

fn main() {
  let mut v = StaticVec::<&f32, 24>::new();
  for _i in 0..v.capacity() {
    v.push(&24.5);
  }
  for f in &v {
    println!("{}", f);
  }
  v.clear();
  v.insert(0, &47.6);
  v.insert(1, &48.6);
  v.insert(2, &49.6);
  v.insert(v.len() - 1, &50.6);
  v.insert(v.len() - 2, &51.6);
  v.insert(v.len() - 3, &52.6);
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
  let mut va = StaticVec::<usize, 65536>::new();
  for i in 0..va.capacity() {
    va.push(i);
  }
  va.remove(10);
  va.remove(10);
  va.remove(10);
  va.remove(10);
  va.remove(10);
  va.remove(10);
  va.insert(10, 99);
  va.insert(10, 99);
  va.insert(10, 99);
  va.insert(10, 99);
  va.insert(10, 99);
  va.insert(10, 99);
  for i in 0..va.len() {
    println!("{}", va[i])
  }
  for i in &va {
    println!("{}", i)
  }
  while va.is_not_empty() {
    println!("{}", va.pop().unwrap());
  }
  let mut vb = StaticVec::<&'static str, 24>::new();
  vb.push("a");
  vb.push("b");
  vb.push("c");
  vb.push("d");
  vb.push("e");
  vb.push("f");
  vb.push("g");
  vb.push("h");
  vb.remove(2);
  vb.remove(2);
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
    vb.push("hello");
  }
  while vb.is_not_empty() {
    println!("{}", vb.remove(0));
  }
  vb.push("g");
  vb.push("f");
  vb.push("e");
  vb.push("d");
  vb.push("c");
  vb.push("b");
  vb.push("a");
  let vbm = vb.as_mut_slice();
  vbm.sort();
  for s in vbm {
    println!("{}", s);
  }
  let vbmb = vb.as_mut_slice();
  vbmb.reverse();
  for s in vbmb {
    println!("{}", s);
  }
  for s in &vb.sorted() {
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
  let vvu = StaticVec::<usize, 4> = vu.iter.collect();
  for i in vvu {
    println!("{}", i);
  }
  for i in vu.drain(2..5).iter().find(|&&i| i == 4) {
    println!("{}", i);
  }
  bloop();
  bloop();
  bloop();
  bloop();
  let mut empty = StaticVec::<&'static str, 0>::new();
  empty.sort();
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
  for s in &empty.sorted() {
    println!("{}", s);
  }
  let mut msv = StaticVec::<MyStruct, 4>::new();
  msv.push(MyStruct { s: "a" });
  msv.push(MyStruct { s: "b" });
  msv.push(MyStruct { s: "c" });
  msv.push(MyStruct { s: "d" });
  msv.clear();
  msv.push(MyStruct { s: "a" });
  msv.push(MyStruct { s: "b" });
  msv.push(MyStruct { s: "c" });
  msv.push(MyStruct { s: "d" });
  for ms in &msv {
    println!("{}", ms.s);
  }
}
