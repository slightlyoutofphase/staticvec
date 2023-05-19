use core::char::DecodeUtf16Error;
use core::fmt::{self, Debug, Display, Formatter};
use core::str::Utf8Error;

use crate::errors::CapacityError;

/// This enum represents several different possible "error states" that may be encountered
/// while using a [`StaticString`](crate::string::StaticString).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringError {
  /// Indicates a failed conversion from a `u8` slice to a
  /// [`StaticString`](crate::string::StaticString).
  Utf8(Utf8Error),
  /// Indicates a failed conversion from a `u16` slice to a
  /// [`StaticString`](crate::string::StaticString).
  Utf16(DecodeUtf16Error),
  /// Indicates an attempted access of an invalid UTF-8 character index.
  NotCharBoundary,
  /// Indicates an out-of-bounds indexed access of a [`StaticString`](crate::string::StaticString)
  /// instance.
  OutOfBounds,
}

#[allow(clippy::match_like_matches_macro)]
impl StringError {
  #[inline(always)]
  pub const fn is_utf8(&self) -> bool {
    match self {
      Self::Utf8(_) => true,
      _ => false,
    }
  }

  #[inline(always)]
  pub const fn is_utf16(&self) -> bool {
    match self {
      Self::Utf16(_) => true,
      _ => false,
    }
  }

  #[inline(always)]
  pub const fn is_out_of_bounds(&self) -> bool {
    match self {
      Self::OutOfBounds => true,
      _ => false,
    }
  }

  #[inline(always)]
  pub const fn is_not_char_boundary(&self) -> bool {
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
#[doc(cfg(feature = "std"))]
impl std::error::Error for StringError {}

impl const From<DecodeUtf16Error> for StringError {
  #[inline(always)]
  fn from(err: DecodeUtf16Error) -> Self {
    Self::Utf16(err)
  }
}

impl const From<Utf8Error> for StringError {
  #[inline(always)]
  fn from(err: Utf8Error) -> Self {
    Self::Utf8(err)
  }
}

impl<const N: usize> const From<CapacityError<N>> for StringError {
  #[inline(always)]
  fn from(_err: CapacityError<N>) -> Self {
    Self::OutOfBounds
  }
}
