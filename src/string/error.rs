use core::char::DecodeUtf16Error;
use core::fmt::{self, Debug, Display, Formatter};
use core::str::Utf8Error;

/// This enum represents several different possible "error states" that may be encountered
/// while using a `StaticString`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringError {
  /// Conversion between available byte slice and UTF-8 failed
  Utf8(Utf8Error),
  /// Conversion between available `u16` slice and string failed
  Utf16(DecodeUtf16Error),
  /// Accessed invalid Utf8 character index
  NotCharBoundary,
  /// Out of boundaries access
  OutOfBounds,
}

impl StringError {
  #[inline(always)]
  pub fn is_utf8(&self) -> bool {
    match self {
      Self::Utf8(_) => true,
      _ => false,
    }
  }

  #[inline(always)]
  pub fn is_utf16(&self) -> bool {
    match self {
      Self::Utf16(_) => true,
      _ => false,
    }
  }

  #[inline(always)]
  pub fn is_out_of_bounds(&self) -> bool {
    match self {
      Self::OutOfBounds => true,
      _ => false,
    }
  }

  #[inline(always)]
  pub fn is_not_char_boundary(&self) -> bool {
    match self {
      Self::NotCharBoundary => true,
      _ => false,
    }
  }
}

impl Display for StringError {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Utf8(err) => write!(f, "{}", err),
      Self::Utf16(err) => write!(f, "{}", err),
      Self::OutOfBounds => write!(f, "Out Of Bounds"),
      Self::NotCharBoundary => write!(f, "Not Char Boundary"),
    }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for StringError {}

impl From<DecodeUtf16Error> for StringError {
  #[inline(always)]
  fn from(err: DecodeUtf16Error) -> Self {
    Self::Utf16(err)
  }
}

impl From<Utf8Error> for StringError {
  #[inline(always)]
  fn from(err: Utf8Error) -> Self {
    Self::Utf8(err)
  }
}
