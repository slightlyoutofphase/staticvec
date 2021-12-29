use super::{StaticString, StringError};

/// Unsafely marks a branch as unreachable.
#[inline(always)]
#[allow(unused_variables)]
pub(crate) unsafe fn never(s: &str) -> ! {
  #[cfg(debug_assertions)]
  core::panic!("{}", s);
  #[cfg(not(debug_assertions))]
  core::hint::unreachable_unchecked()
}

// UTF-8 ranges and tags for encoding characters.
const TAG_CONT: u8 = 0b1000_0000;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;
const MAX_ONE_B: u32 = 0x80;
const MAX_TWO_B: u32 = 0x800;
const MAX_THREE_B: u32 = 0x10000;

/// Encodes `character` into `string` at the specified position.
#[inline(always)]
pub(crate) const unsafe fn encode_char_utf8_unchecked<const N: usize>(
  string: &mut StaticString<N>,
  character: char,
  index: usize,
) {
  let dest = string.vec.mut_ptr_at_unchecked(index);
  let code = character as u32;
  if code < MAX_ONE_B {
    dest.write(code as u8);
    string.vec.set_len(string.len() + 1);
  } else if code < MAX_TWO_B {
    dest.write((code >> 6 & 0x1F) as u8 | TAG_TWO_B);
    dest.offset(1).write((code & 0x3F) as u8 | TAG_CONT);
    string.vec.set_len(string.len() + 2);
  } else if code < MAX_THREE_B {
    dest.write((code >> 12 & 0x0F) as u8 | TAG_THREE_B);
    dest.offset(1).write((code >> 6 & 0x3F) as u8 | TAG_CONT);
    dest.offset(2).write((code & 0x3F) as u8 | TAG_CONT);
    string.vec.set_len(string.len() + 3);
  } else {
    dest.write((code >> 18 & 0x07) as u8 | TAG_FOUR_B);
    dest.offset(1).write((code >> 12 & 0x3F) as u8 | TAG_CONT);
    dest.offset(2).write((code >> 6 & 0x3F) as u8 | TAG_CONT);
    dest.offset(3).write((code & 0x3F) as u8 | TAG_CONT);
    string.vec.set_len(string.len() + 4);
  }
}

/// Shifts `string` to the right.
#[inline(always)]
pub(crate) unsafe fn shift_right_unchecked<const N: usize>(
  string: &mut StaticString<N>,
  from: usize,
  to: usize,
) {
  debug_assert!(from <= to && to + string.len() - from <= string.capacity());
  debug_assert!(string.as_str().is_char_boundary(from));
  string
    .as_ptr()
    .add(from)
    .copy_to(string.as_mut_ptr().add(to), string.len() - from);
}

/// Shifts `string` to the left.
#[inline(always)]
pub(crate) unsafe fn shift_left_unchecked<const N: usize>(
  string: &mut StaticString<N>,
  from: usize,
  to: usize,
) {
  debug_assert!(to <= from && from <= string.len());
  debug_assert!(string.as_str().is_char_boundary(from));
  string
    .as_ptr()
    .add(from)
    .copy_to(string.as_mut_ptr().add(to), string.len() - from);
}

/// Returns an error if `size` is greater than `limit`.
#[inline(always)]
pub(crate) const fn is_inside_boundary(size: usize, limit: usize) -> Result<(), StringError> {
  match size <= limit {
    false => Err(StringError::OutOfBounds),
    true => Ok(()),
  }
}

/// Returns an error if `index` is not at a valid UTF-8 character boundary.
#[inline(always)]
pub(crate) fn is_char_boundary<const N: usize>(
  string: &StaticString<N>,
  index: usize,
) -> Result<(), StringError> {
  match string.as_str().is_char_boundary(index) {
    false => Err(StringError::NotCharBoundary),
    true => Ok(()),
  }
}

/// Truncates `slice` to the specified size (ignoring the last few bytes if they form a partial
/// `char`).
#[inline]
pub(crate) fn truncate_str(slice: &str, size: usize) -> &str {
  if slice.is_char_boundary(size) {
    unsafe { slice.get_unchecked(..size) }
  } else if size < slice.len() {
    let mut index = size - 1;
    while !slice.is_char_boundary(index) {
      index -= 1;
    }
    unsafe { slice.get_unchecked(..index) }
  } else {
    slice
  }
}

/// Macro to avoid code duplication in char-pushing methods.
macro_rules! push_char_unchecked_internal {
  ($self_var:expr, $char_var:expr, $len:expr) => {
    #[allow(unused_unsafe)]
    match $len {
      1 => unsafe { $self_var.vec.push_unchecked($char_var as u8) },
      _ => {
        let old_length = $self_var.len();
        unsafe {
          $char_var
            .encode_utf8(&mut [0; 4])
            .as_ptr()
            .copy_to_nonoverlapping($self_var.vec.mut_ptr_at_unchecked(old_length), $len);
          $self_var.vec.set_len(old_length + $len);
        }
      }
    };
  };
}

/// Macro to avoid code duplication in char-pushing methods.
macro_rules! push_str_unchecked_internal {
  ($self_var:expr, $str_var:expr, $self_len_var:expr, $str_len_var:expr) => {
    #[allow(unused_unsafe)]
    unsafe {
      let dest = $self_var.vec.mut_ptr_at_unchecked($self_len_var);
      $str_var.as_ptr().copy_to_nonoverlapping(dest, $str_len_var);
      $self_var.vec.set_len($self_len_var + $str_len_var);
    }
  };
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
