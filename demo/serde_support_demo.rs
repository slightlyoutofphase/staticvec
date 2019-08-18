use serde::{Deserialize, Serialize};
use staticvec::*;

#[derive(Serialize, Deserialize)]
struct MyStruct {
  value: &'static str,
}

fn main() {
  let structs = staticvec![
    MyStruct { value: "Serde" },
    MyStruct { value: "sure" },
    MyStruct { value: "makes" },
    MyStruct { value: "this" },
    MyStruct { value: "really" },
    MyStruct { value: "easy!" },
  ];
  println!(
    "{}",
    serde_json::to_string_pretty(&structs).expect("You should never see this!")
  );
}
