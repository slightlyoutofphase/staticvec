use serde::{Deserialize, Serialize};

use staticvec::{staticvec, StaticHeap, StaticString, StaticVec};

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

  let string_json = serde_json::to_string_pretty(&StaticString::<8>::from("abcdefg")).unwrap();

  let static_string: StaticString<8> = serde_json::from_str(&string_json).unwrap();

  let heap_json = serde_json::to_string_pretty(&StaticHeap::from([1, 2, 3, 4, 5])).unwrap();

  println!("{} \n\n{:?}\n\n{}", string_json, static_string, heap_json);
}
