use serde::{Deserialize, Serialize};
use staticvec::*;

#[derive(Debug, Deserialize, Serialize)]
struct MyStruct {
  value: &'static str,
}

const JSON_STR: &str = r#"
[
  {
    "value": "easy!"
  },
  {
    "value": "really"
  },
  {
    "value": "this"
  },
  {
    "value": "makes"
  },
  {
    "value": "sure"
  },
  {
    "value": "Serde"
  }
]
"#;

fn main() {
  let structs_a = staticvec![
    MyStruct { value: "Serde" },
    MyStruct { value: "sure" },
    MyStruct { value: "makes" },
    MyStruct { value: "this" },
    MyStruct { value: "really" },
    MyStruct { value: "easy!" },
  ];

  let structs_b: StaticVec<MyStruct, 6> = serde_json::from_str(JSON_STR).unwrap();

  println!(
    "{} \n\n{:?}\n",
    serde_json::to_string_pretty(&structs_a).unwrap(),
    structs_b
  );

  let json = serde_json::to_string_pretty(&StaticString::<8>::from("abcdefg")).unwrap();

  let string: StaticString<8> = serde_json::from_str(&json).unwrap();

  println!("{} \n\n{:?}", json, string);
}
