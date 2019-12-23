//! Misc functions to improve readability

use super::{StaticString, StaticStringError};
use core::ptr::{copy, write};

pub(crate) trait IntoLossy<T>: Sized {
  fn into_lossy(self) -> T;
}

/// Marks branch as impossible, UB if taken in prod, panics in debug
///
/// This function should never be used lightly, it will cause UB if used wrong
#[inline]
#[allow(unused_variables)]
pub(crate) unsafe fn never(s: &str) -> ! {
  #[cfg(debug_assertions)]
  panic!("{}", s);

  #[cfg(not(debug_assertions))]
  core::hint::unreachable_unchecked()
}

/// Encodes `char` into `StaticString` at specified position, heavily unsafe
///
/// We reimplement the `core` function to avoid panicking (UB instead, be careful)
///
/// Reimplemented from;
///
/// `https://github.com/rust-lang/rust/blob/7843e2792dce0f20d23b3c1cca51652013bef0ea/src/libcore/char/methods.rs#L447`
/// # Safety
///
/// - It's UB if index is outside of buffer's boundaries (buffer needs at most 4 bytes)
/// - It's UB if index is inside a character (like a index 3 for "a🤔")
#[inline]
pub(crate) unsafe fn encode_char_utf8_unchecked<const N: usize>(
  s: &mut StaticString<N>,
  ch: char,
  index: usize,
)
{
  // UTF-8 ranges and tags for encoding characters
  #[allow(clippy::missing_docs_in_private_items)]
  const TAG_CONT: u8 = 0b1000_0000;
  #[allow(clippy::missing_docs_in_private_items)]
  const TAG_TWO_B: u8 = 0b1100_0000;
  #[allow(clippy::missing_docs_in_private_items)]
  const TAG_THREE_B: u8 = 0b1110_0000;
  #[allow(clippy::missing_docs_in_private_items)]
  const TAG_FOUR_B: u8 = 0b1111_0000;
  #[allow(clippy::missing_docs_in_private_items)]
  const MAX_ONE_B: u32 = 0x80;
  #[allow(clippy::missing_docs_in_private_items)]
  const MAX_TWO_B: u32 = 0x800;
  #[allow(clippy::missing_docs_in_private_items)]
  const MAX_THREE_B: u32 = 0x10000;

  debug_assert!(ch.len_utf8().saturating_add(index) <= s.capacity());
  debug_assert!(ch.len_utf8().saturating_add(s.len()) <= s.capacity());
  let dst = s.vec.as_mut_ptr().add(index);
  let code = ch as u32;

  if code < MAX_ONE_B {
    debug_assert!(N.saturating_sub(index) >= 1);
    write(dst, code.into_lossy());
    s.vec.set_len(s.len().saturating_add(1));
  } else if code < MAX_TWO_B {
    debug_assert!(N.saturating_sub(index) >= 2);
    write(dst, (code >> 6 & 0x1F).into_lossy() | TAG_TWO_B);
    write(dst.add(1), (code & 0x3F).into_lossy() | TAG_CONT);
    s.vec.set_len(s.len().saturating_add(2));
  } else if code < MAX_THREE_B {
    debug_assert!(N.saturating_sub(index) >= 3);
    write(dst, (code >> 12 & 0x0F).into_lossy() | TAG_THREE_B);
    write(dst.add(1), (code >> 6 & 0x3F).into_lossy() | TAG_CONT);
    write(dst.add(2), (code & 0x3F).into_lossy() | TAG_CONT);
    s.vec.set_len(s.len().saturating_add(3));
  } else {
    debug_assert!(N.saturating_sub(index) >= 4);
    write(dst, (code >> 18 & 0x07).into_lossy() | TAG_FOUR_B);
    write(dst.add(1), (code >> 12 & 0x3F).into_lossy() | TAG_CONT);
    write(dst.add(2), (code >> 6 & 0x3F).into_lossy() | TAG_CONT);
    write(dst.add(3), (code & 0x3F).into_lossy() | TAG_CONT);
    s.vec.set_len(s.len().saturating_add(4));
  }
}

/// Shifts string right
///
/// # Safety
///
/// It's UB if `to + (s.len() - from)` is bigger than [`S::to_usize()`]
///
/// [`<S as Unsigned>::to_usize()`]: ../struct.StaticString.html#CAPACITY
#[inline]
pub(crate) unsafe fn shift_right_unchecked<const N: usize>(
  s: &mut StaticString<N>,
  from: usize,
  to: usize,
)
{
  let len = s.len().saturating_sub(from);
  debug_assert!(from <= to && to.saturating_add(len) <= s.capacity());
  debug_assert!(s.as_str().is_char_boundary(from));
  copy(
    s.as_ptr().add(from),
    s.as_mut_ptr().add(to),
    s.len().saturating_sub(from),
  );
}

/// Shifts string left
#[inline]
pub(crate) unsafe fn shift_left_unchecked<const N: usize>(
  s: &mut StaticString<N>,
  from: usize,
  to: usize,
)
{
  debug_assert!(to <= from && from <= s.len());
  debug_assert!(s.as_str().is_char_boundary(from));
  copy(
    s.as_ptr().add(from),
    s.as_mut_ptr().add(to),
    s.len().saturating_sub(from),
  );
}

/// Returns error if size is outside of specified boundary
#[inline]
pub fn is_inside_boundary(size: usize, limit: usize) -> Result<(), StaticStringError> {
  Some(())
    .filter(|_| size <= limit)
    .ok_or(StaticStringError::OutOfBounds)
}

/// Returns error if index is not at a valid utf-8 char boundary
#[inline]
pub fn is_char_boundary<const N: usize>(
  s: &StaticString<N>,
  idx: usize,
) -> Result<(), StaticStringError>
{
  if s.as_str().is_char_boundary(idx) {
    return Ok(());
  }
  Err(StaticStringError::NotCharBoundary)
}

/// Truncates string to specified size (ignoring last bytes if they form a partial `char`)
#[inline]
pub(crate) fn truncate_str(slice: &str, size: usize) -> &str {
  if slice.is_char_boundary(size) {
    unsafe { slice.get_unchecked(..size) }
  } else if size < slice.len() {
    let mut index = size.saturating_sub(1);
    while !slice.is_char_boundary(index) {
      index = index.saturating_sub(1);
    }
    unsafe { slice.get_unchecked(..index) }
  } else {
    slice
  }
}

impl IntoLossy<u8> for usize {
  #[allow(clippy::cast_possible_truncation)]
  #[inline]
  fn into_lossy(self) -> u8 {
    self as u8
  }
}

impl IntoLossy<u8> for u32 {
  #[allow(clippy::cast_possible_truncation)]
  #[inline]
  fn into_lossy(self) -> u8 {
    self as u8
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use core::str::from_utf8;

  #[test]
  fn truncate() {
    assert_eq!(truncate_str("i", 10), "i");
    assert_eq!(truncate_str("iiiiii", 3), "iii");
    assert_eq!(truncate_str("🤔🤔🤔", 5), "🤔");
  }

  #[test]
  fn shift_right() {
    let mut ls = StaticString::<20>::try_from_str("abcdefg").unwrap();
    unsafe { shift_right_unchecked(&mut ls, 0usize, 4usize) };
    unsafe { ls.vec.set_len(ls.len() + 4) };
    assert_eq!(ls.as_str(), "abcdabcdefg");
  }

  #[test]
  fn shift_left() {
    let mut ls = StaticString::<20>::try_from_str("abcdefg").unwrap();
    unsafe { shift_left_unchecked(&mut ls, 1usize, 0usize) };
    unsafe { ls.vec.set_len(ls.len() - 1) };
    assert_eq!(ls.as_str(), "bcdefg");
  }

  #[test]
  fn shift_nop() {
    let mut ls = StaticString::<20>::try_from_str("abcdefg").unwrap();
    unsafe { shift_right_unchecked(&mut ls, 0usize, 0usize) };
    assert_eq!(ls.as_str(), "abcdefg");
    unsafe { shift_left_unchecked(&mut ls, 0usize, 0usize) };
    assert_eq!(ls.as_str(), "abcdefg");
  }

  #[test]
  fn encode_char_utf8() {
    let mut string = StaticString::<20>::default();
    unsafe {
      encode_char_utf8_unchecked(&mut string, 'a', 0);
      assert_eq!(from_utf8(&string.as_mut_bytes()).unwrap(), "a");
      let mut string = StaticString::<20>::try_from_str("a").unwrap();

      encode_char_utf8_unchecked(&mut string, '🤔', 1);
      assert_eq!(from_utf8(&string.as_mut_bytes()[..5]).unwrap(), "a🤔");
    }
  }
}
