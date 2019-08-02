use staticvec::*;

#[derive(Copy, Clone)]
struct MyStruct {
  s: &'static str,
}

fn main() {
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
  let mut va = StaticVec::<usize, 65536>::new();
  for i in 0..va.capacity() {
    va.push(i);
  }
  let ia = va.remove_item(&32768).unwrap();
  let ib = va.remove_item(&32767).unwrap();
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
    staticvec![24.0, 26.0]
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
}
