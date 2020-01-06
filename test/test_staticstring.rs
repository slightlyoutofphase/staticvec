#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(incomplete_features)]
#![allow(unused_imports)]
#![feature(const_fn, const_generics, const_if_match, const_loop)]

use staticvec::*;

type MyString = StaticString<255>;

#[test]
fn test_push_bytes() {
  let mut s = MyString::from("ABC");
  let mv = unsafe { s.as_mut_staticvec() };
  mv.extend_from_slice(&[b'D']);
  assert_eq!(s, "ABCD");
}

#[test]
fn test_from_utf8() {
  let xs = b"hello".to_vec();
  assert_eq!(MyString::from_utf8(xs).unwrap(), MyString::from("hello"));
  let xs = "ศไทย中华Việt Nam".as_bytes().to_vec();
  assert_eq!(MyString::from_utf8(xs).unwrap(), MyString::from("ศไทย中华Việt Nam"));
  let xs = b"hello\xFF".to_vec();
  let err = MyString::from_utf8(xs);
  assert!(err.is_err());
}

#[test]
fn test_from_utf16() {
  type MyStaticVec = StaticVec<u16, 255>;
  let pairs = [
    (
      MyString::from("𐍅𐌿𐌻𐍆𐌹𐌻𐌰\n"),
      MyStaticVec::from([
        0xd800, 0xdf45, 0xd800, 0xdf3f, 0xd800, 0xdf3b, 0xd800, 0xdf46, 0xd800, 0xdf39,
        0xd800, 0xdf3b, 0xd800, 0xdf30, 0x000a,
      ]),
    ),
    (
      MyString::from("𐐒𐑉𐐮𐑀𐐲𐑋 𐐏𐐲𐑍\n"),
      MyStaticVec::from([
        0xd801, 0xdc12, 0xd801, 0xdc49, 0xd801, 0xdc2e, 0xd801, 0xdc40, 0xd801, 0xdc32,
        0xd801, 0xdc4b, 0x0020, 0xd801, 0xdc0f, 0xd801, 0xdc32, 0xd801, 0xdc4d, 0x000a,
      ]),
    ),
    (
      MyString::from("𐌀𐌖𐌋𐌄𐌑𐌉·𐌌𐌄𐌕𐌄𐌋𐌉𐌑\n"),
      MyStaticVec::from([
        0xd800, 0xdf00, 0xd800, 0xdf16, 0xd800, 0xdf0b, 0xd800, 0xdf04, 0xd800, 0xdf11,
        0xd800, 0xdf09, 0x00b7, 0xd800, 0xdf0c, 0xd800, 0xdf04, 0xd800, 0xdf15, 0xd800,
        0xdf04, 0xd800, 0xdf0b, 0xd800, 0xdf09, 0xd800, 0xdf11, 0x000a,
      ]),
    ),
    (
      MyString::from("𐒋𐒘𐒈𐒑𐒛𐒒 𐒕𐒓 𐒈𐒚𐒍 𐒏𐒜𐒒𐒖𐒆 𐒕𐒆\n"),
      MyStaticVec::from([
        0xd801, 0xdc8b, 0xd801, 0xdc98, 0xd801, 0xdc88, 0xd801, 0xdc91, 0xd801, 0xdc9b,
        0xd801, 0xdc92, 0x0020, 0xd801, 0xdc95, 0xd801, 0xdc93, 0x0020, 0xd801, 0xdc88,
        0xd801, 0xdc9a, 0xd801, 0xdc8d, 0x0020, 0xd801, 0xdc8f, 0xd801, 0xdc9c, 0xd801,
        0xdc92, 0xd801, 0xdc96, 0xd801, 0xdc86, 0x0020, 0xd801, 0xdc95, 0xd801, 0xdc86,
        0x000a,
      ]),
    ),
    (MyString::from("\u{20000}"), MyStaticVec::from([0xD840, 0xDC00])),
  ];
  for p in &pairs {
    let (s, u) = (*p).clone();
    let s_as_utf16 = s.encode_utf16().collect::<StaticVec<u16, 255>>();
    let u_as_string = MyString::from_utf16(&u).unwrap();
    assert!(core::char::decode_utf16(u.iter().cloned()).all(|r| r.is_ok()));
    assert_eq!(s_as_utf16, u);
    assert_eq!(u_as_string, s);
    assert_eq!(MyString::from_utf16_lossy(&u), s);
    assert_eq!(MyString::from_utf16(&s_as_utf16).unwrap(), s);
    assert_eq!(u_as_string.encode_utf16().collect::<StaticVec<u16, 255>>(), u);
  }
}

#[test]
fn test_push_str() {
  let mut s = MyString::new();
  s.push_str("");
  assert_eq!(&s[0..], "");
  s.push_str("abc");
  assert_eq!(&s[0..], "abc");
  s.push_str("ประเทศไทย中华Việt Nam");
  assert_eq!(&s[0..], "abcประเทศไทย中华Việt Nam");
}

#[test]
fn test_add_assign() {
  let mut s = MyString::new();
  s += "";
  assert_eq!(s.as_str(), "");
  s += "abc";
  assert_eq!(s.as_str(), "abc");
  s += "ประเทศไทย中华Việt Nam";
  assert_eq!(s.as_str(), "abcประเทศไทย中华Việt Nam");
}

#[test]
fn test_push() {
  let mut data = MyString::from("ประเทศไทย中");
  data.push('华');
  data.push('b'); // 1 byte
  data.push('¢'); // 2 byte
  data.push('€'); // 3 byte
  data.push('𤭢'); // 4 byte
  assert_eq!(data, "ประเทศไทย中华b¢€𤭢");
}

#[test]
fn test_pop() {
  let mut data = MyString::from("ประเทศไทย中华b¢€𤭢");
  assert_eq!(data.pop().unwrap(), '𤭢'); // 4 bytes
  assert_eq!(data.pop().unwrap(), '€'); // 3 bytes
  assert_eq!(data.pop().unwrap(), '¢'); // 2 bytes
  assert_eq!(data.pop().unwrap(), 'b'); // 1 bytes
  assert_eq!(data.pop().unwrap(), '华');
  assert_eq!(data, "ประเทศไทย中");
}

#[test]
fn test_split_off_empty() {
  let orig = "Hello, world!";
  let mut split = MyString::from(orig);
  let empty = split.split_off(orig.len());
  assert!(empty.is_empty());
}

#[test]
#[should_panic]
fn test_split_off_past_end() {
  let orig = "Hello, world!";
  let mut split = MyString::from(orig);
  split.split_off(orig.len() + 1);
}

#[test]
#[should_panic]
fn test_split_off_mid_char() {
  let mut orig = MyString::from("山");
  orig.split_off(1);
}

#[test]
fn test_split_off_ascii() {
  let mut ab = MyString::from("ABCD");
  let cd = ab.split_off(2);
  assert_eq!(ab, "AB");
  assert_eq!(cd, "CD");
}

#[test]
fn test_split_off_unicode() {
  let mut nihon = MyString::from("日本語");
  let go = nihon.split_off("日本".len());
  assert_eq!(nihon, "日本");
  assert_eq!(go, "語");
}

#[test]
fn test_str_truncate() {
  let mut s = MyString::from("12345");
  s.truncate(5).unwrap();
  assert_eq!(s, "12345");
  s.truncate(3).unwrap();
  assert_eq!(s, "123");
  s.truncate(0).unwrap();
  assert_eq!(s, "");
  let mut s = MyString::from("12345");
  let p = s.as_ptr();
  s.truncate(3).unwrap();
  s.push_str("6");
  let p_ = s.as_ptr();
  assert_eq!(p_, p);
}

#[test]
fn test_str_truncate_invalid_len() {
  let mut s = MyString::from("12345");
  s.truncate(6).unwrap();
  assert_eq!(s, "12345");
}

#[test]
#[should_panic]
fn test_str_truncate_split_codepoint() {
  let mut s = MyString::from("\u{FC}"); // ü
  s.truncate(1).unwrap();
}

#[test]
fn test_str_clear() {
  let mut s = MyString::from("12345");
  s.clear();
  assert_eq!(s.len(), 0);
  assert_eq!(s, "");
}

#[test]
fn test_str_add() {
  let a = MyString::from("12345");
  let b = a + "2";
  let b = b + "2";
  assert_eq!(b.len(), 7);
  assert_eq!(b, "1234522");
}

#[test]
fn remove() {
  let mut s = MyString::from("ศไทย中华Việt Nam; foobar");
  assert_eq!(s.remove(0), 'ศ');
  assert_eq!(s.len(), 33);
  assert_eq!(s, "ไทย中华Việt Nam; foobar");
  assert_eq!(s.remove(17), 'ệ');
  assert_eq!(s, "ไทย中华Vit Nam; foobar");
}

#[test]
#[should_panic]
fn remove_bad() {
  StaticString::<0>::from("ศ").remove(1);
}

#[test]
fn test_retain() {
  let mut s = MyString::from("α_β_γ");
  s.retain(|_| true);
  assert_eq!(s, "α_β_γ");
  s.retain(|c| c != '_');
  assert_eq!(s, "αβγ");
  s.retain(|c| c != 'β');
  assert_eq!(s, "αγ");
  s.retain(|c| c == 'α');
  assert_eq!(s, "α");
  s.retain(|_| false);
  assert_eq!(s, "");
}

#[test]
fn insert() {
  let mut s = MyString::from("foobar");
  s.insert(0, 'ệ');
  assert_eq!(s, "ệfoobar");
  s.insert(6, 'ย');
  assert_eq!(s, "ệfooยbar");
}

#[test]
#[should_panic]
fn insert_bad1() {
  StaticString::<0>::from("").insert(1, 't');
}

#[test]
#[should_panic]
fn insert_bad2() {
  StaticString::<0>::from("ệ").insert(1, 't');
}

#[test]
fn test_slicing() {
  let s = MyString::from("foobar");
  assert_eq!(&s[..], "foobar");
  assert_eq!(&s[..3], "foo");
  assert_eq!(&s[3..], "bar");
  assert_eq!(&s[1..4], "oob");
}

#[test]
fn test_from_iterator() {
  let s = "ศไทย中华Việt Nam";
  let t = "ศไทย中华";
  let u = "Việt Nam";
  let a: MyString = s.chars().collect();
  assert_eq!(a, s);
  let mut b = MyString::from(t);
  b.extend(u.chars());
  assert_eq!(b, s);
  let c: MyString = staticvec![t, u].into_iter().collect();
  assert_eq!(c, s);
  let mut d = MyString::from(t);
  d.extend(staticvec![u]);
  assert_eq!(d, s);
}

#[test]
fn test_replace_range() {
  let mut s = MyString::from("Hello, world!");
  s.replace_range(7..12, "世界").unwrap();
  assert_eq!(s, "Hello, 世界!");
}

#[test]
#[should_panic]
fn test_replace_range_char_boundary() {
  let mut s = MyString::from("Hello, 世界!");
  s.replace_range(..8, "").unwrap();
}

#[test]
fn test_replace_range_inclusive_range() {
  let mut v = MyString::from("12345");
  v.replace_range(2..=3, "789").unwrap();
  assert_eq!(v, "127895");
  v.replace_range(1..=2, "A").unwrap();
  assert_eq!(v, "1A895");
}

#[test]
#[should_panic]
fn test_replace_range_out_of_bounds() {
  let mut s = MyString::from("12345");
  s.replace_range(5..6, "789").unwrap();
}

#[test]
#[should_panic]
fn test_replace_range_inclusive_out_of_bounds() {
  let mut s = MyString::from("12345");
  s.replace_range(5..=5, "789").unwrap();
}

#[test]
fn test_replace_range_empty() {
  let mut s = MyString::from("12345");
  s.replace_range(1..2, "").unwrap();
  assert_eq!(s, "1345");
}

#[test]
fn test_replace_range_unbounded() {
  let mut s = MyString::from("12345");
  s.replace_range(.., "").unwrap();
  assert_eq!(s, "");
}
