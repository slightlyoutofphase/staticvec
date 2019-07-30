#![feature(duration_float)]

use staticvec::StaticVec;
use std::time::Instant;

fn main() {
  let instanta = Instant::now();
  let mut sv = StaticVec::<usize, 262144>::new();
  for i in 0..262144 {
    sv.push(i);
  }
  while sv.len() > 0 {
    println!("{}", sv.pop().unwrap());
  }
  println!("{}", instanta.elapsed().as_secs_f64());
}
