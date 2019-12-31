// So we don't get "function complexity" lints and such since it's a demo.
#![allow(clippy::all)]

use staticvec::*;

fn main() {
  let mut s = StaticString::<4>::new();
  s.push_str("🤔");
  println!("Value: {}", s);
  println!("Debug info: {:?}", s);
  println!("Length: {}", s.len());
  println!("Remaining capacity: {}", s.remaining_capacity());
  let mut s2 = StaticString::<4>::new();
  s2.push('🤔');
  println!("Value: {}", s2);
  println!("Debug info: {:?}", s2);
  println!("Length: {}", s2.len());
  println!("Remaining capacity: {}", s2.remaining_capacity());
  let s3 = StaticString::<7>::from_chars(
    staticvec!['A', 'B', 'C', 'D', 'E', 'F', 'G']
      .reversed()
      .into_iter(),
  );
  println!("Value: {}", s3);
  println!("Debug info: {:?}", s3);
  println!("Length: {}", s3.len());
  println!("Remaining capacity: {}", s3.remaining_capacity());
  let v = staticvec!["ABCDEFG", "HIJKLMNOP", "QRSTUV", "WXYZ"];
  let s4 = StaticString::<26>::from_iter(v.into_iter());
  println!("Value: {}", s4);
  println!("Debug info: {:?}", s4);
  println!("Length: {}", s4.len());
  println!("Remaining capacity: {}", s4.remaining_capacity());
  let mut s5 = StaticString::<6>::from("ABEF");
  s5.insert_str(2, "CD").unwrap();
  println!("Value: {}", s5);
  println!("Debug info: {:?}", s5);
  println!("Length: {}", s5.len());
  println!("Remaining capacity: {}", s5.remaining_capacity());
  s5.replace_range(2..4, "XY").unwrap();
  println!("Value: {}", s5);
  println!("Debug info: {:?}", s5);
  println!("Length: {}", s5.len());
  println!("Remaining capacity: {}", s5.remaining_capacity());
}
