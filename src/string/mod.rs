//! A fixed-capacity `String`-like struct built around a `StaticVec<u8, N>`.
//!
//! [`StaticVec`]: ../struct.StaticVec.html
//!
//! ## Examples
//!
//! ```rust
//! use staticvec::{StaticString, StaticStringError};
//!
//! #[derive(Debug)]
//! pub struct User {
//!   pub username: StaticString<20>,
//!   pub role: StaticString<5>,
//! }
//!
//! fn main() -> Result<(), StaticStringError> {
//!   let user = User {
//!     username: StaticString::try_from_str("user")?,
//!     role: StaticString::try_from_str("admin")?,
//!   };
//!   println!("{:?}", user);
//!   Ok(())
//! }
//! ```

pub use self::error::StaticStringError;
use self::utils::{
  encode_char_utf8_unchecked, is_char_boundary, is_inside_boundary, never, shift_left_unchecked,
  shift_right_unchecked, truncate_str,
};
use crate::StaticVec;
use core::char::{decode_utf16, REPLACEMENT_CHARACTER};
use core::cmp::min;
use core::ops::*;
use core::str::{self, from_utf8, from_utf8_unchecked};

mod error;
mod trait_impls;
#[doc(hidden)]
pub mod utils;

/// A fixed-capacity [`String`](alloc::string::String)-like struct built around a `StaticVec<u8,
/// N>`.
///
/// [`StaticVec`]: ../struct.StaticVec.html
#[derive(Clone)]
pub struct StaticString<const N: usize> {
  pub(crate) vec: StaticVec<u8, N>,
}

impl<const N: usize> StaticString<N> {
  /// Creates a new empty string.
  ///
  /// ```rust
  /// use staticvec::StaticString;
  /// let string = StaticString::<20>::new();
  /// assert!(string.is_empty());
  /// ```
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      vec: StaticVec::new(),
    }
  }

  /// Creates a new `StaticString` from a string slice if the slice has a length less than or equal
  /// to the StaticString's declared capacity, or returns a a
  /// [`StaticStringError`](self::error::StaticStringError) otherwise.
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  /// ```rust
  /// use staticvec::{StaticString, StaticStringError};
  ///
  /// fn main() -> Result<(), StaticStringError> {
  ///   let string = StaticString::<20>::try_from_str("My String")?;
  ///   assert_eq!(string.as_str(), "My String");
  ///   assert_eq!(StaticString::<20>::try_from_str("")?.as_str(), "");
  ///   let out_of_bounds = "0".repeat(21);
  ///   assert!(StaticString::<20>::try_from_str(out_of_bounds).is_err());
  ///   Ok(())
  /// }
  /// ```
  #[inline]
  pub fn try_from_str<S>(s: S) -> Result<Self, StaticStringError>
  where S: AsRef<str> {
    let mut string = Self::default();
    string.try_push_str(s)?;
    Ok(string)
  }

  /// Creates new `StaticString` from string slice truncating size if bigger than [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::StaticString;
  /// let string = StaticString::<20>::from_str_truncate("My String");
  /// # assert_eq!(string.as_str(), "My String");
  /// println!("{}", string);
  ///
  /// let truncate = "0".repeat(21);
  /// let truncated = "0".repeat(20);
  /// let string = StaticString::<20>::from_str_truncate(&truncate);
  /// assert_eq!(string.as_str(), truncated);
  /// ```
  #[inline]
  pub fn from_str_truncate<S>(string: S) -> Self
  where S: AsRef<str> {
    let mut s = Self::default();
    s.push_str(string);
    s
  }

  /// Creates new `StaticString` from string slice assuming length is appropriate.
  ///
  /// # Safety
  ///
  /// It's UB if `string.len()` > [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::StaticString;
  /// let filled = "0".repeat(20);
  /// let string = unsafe {
  ///     StaticString::<20>::from_str_unchecked(&filled)
  /// };
  /// assert_eq!(string.as_str(), filled.as_str());
  ///
  /// // Undefined behavior, don't do it
  /// // let out_of_bounds = "0".repeat(21);
  /// // let ub = unsafe { StaticString::<20>::from_str_unchecked(out_of_bounds) };
  /// ```
  #[inline]
  pub unsafe fn from_str_unchecked<S>(string: S) -> Self
  where S: AsRef<str> {
    let mut out = Self::default();
    out.push_str_unchecked(string);
    out
  }

  /// Creates new `StaticString` from string slice iterator if total length is lower or equal to
  /// [`capacity`], otherwise returns an error.
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let string = StaticString::<300>::try_from_iterator(&["My String", " My Other String"][..])?;
  /// assert_eq!(string.as_str(), "My String My Other String");
  ///
  /// let out_of_bounds = (0..100).map(|_| "000");
  /// assert!(StaticString::<20>::try_from_iterator(out_of_bounds).is_err());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_from_iterator<U, I>(iter: I) -> Result<Self, StaticStringError>
  where
    U: AsRef<str>,
    I: IntoIterator<Item = U>, {
    let mut out = Self::default();
    for s in iter {
      out.try_push_str(s)?;
    }
    Ok(out)
  }

  /// Creates new `StaticString` from string slice iterator truncating size if bigger than
  /// [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let string = StaticString::<300>::from_iterator(&["My String", " Other String"][..]);
  /// assert_eq!(string.as_str(), "My String Other String");
  ///
  /// let out_of_bounds = (0..400).map(|_| "000");
  /// let truncated = "0".repeat(20);
  ///
  /// let truncate = StaticString::<20>::from_iterator(out_of_bounds);
  /// assert_eq!(truncate.as_str(), truncated.as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn from_iterator<U, I>(iter: I) -> Self
  where
    U: AsRef<str>,
    I: IntoIterator<Item = U>, {
    let mut out = Self::default();
    for s in iter {
      if out.try_push_str(s.as_ref()).is_err() {
        out.push_str(s);
        break;
      }
    }
    out
  }

  /// Creates new `StaticString` from string slice iterator assuming length is appropriate.
  ///
  /// # Safety
  ///
  /// It's UB if `iter.map(|c| c.len()).sum()` > [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::StaticString;
  /// let string = unsafe {
  ///     StaticString::<300>::from_iterator_unchecked(&["My String", " My Other String"][..])
  /// };
  /// assert_eq!(string.as_str(), "My String My Other String");
  ///
  /// // Undefined behavior, don't do it
  /// // let out_of_bounds = (0..400).map(|_| "000");
  /// // let undefined_behavior = unsafe {
  /// //     StaticString::<20>::from_iterator_unchecked(out_of_bounds)
  /// // };
  /// ```
  #[inline]
  pub unsafe fn from_iterator_unchecked<U, I>(iter: I) -> Self
  where
    U: AsRef<str>,
    I: IntoIterator<Item = U>, {
    let mut out = Self::default();
    for s in iter {
      out.push_str_unchecked(s);
    }
    out
  }

  /// Creates new `StaticString` from char iterator if total length is lower or equal to
  /// [`capacity`], otherwise returns an error.
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let string = StaticString::<20>::try_from_chars("My String".chars())?;
  /// assert_eq!(string.as_str(), "My String");
  ///
  /// let out_of_bounds = "0".repeat(21);
  /// assert!(StaticString::<20>::try_from_chars(out_of_bounds.chars()).is_err());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_from_chars<I>(iter: I) -> Result<Self, StaticStringError>
  where I: IntoIterator<Item = char> {
    let mut out = Self::default();
    for c in iter {
      out.try_push(c)?;
    }
    Ok(out)
  }

  /// Creates new `StaticString` from char iterator truncating size if bigger than [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::StaticString;
  /// let string = StaticString::<20>::from_chars("My String".chars());
  /// assert_eq!(string.as_str(), "My String");
  ///
  /// let out_of_bounds = "0".repeat(21);
  /// let truncated = "0".repeat(20);
  ///
  /// let truncate = StaticString::<20>::from_chars(out_of_bounds.chars());
  /// assert_eq!(truncate.as_str(), truncated.as_str());
  /// ```
  #[inline]
  pub fn from_chars<I>(iter: I) -> Self
  where I: IntoIterator<Item = char> {
    let mut out = Self::default();
    for c in iter {
      if out.try_push(c).is_err() {
        break;
      }
    }
    out
  }

  /// Creates new `StaticString` from char iterator assuming length is appropriate.
  ///
  /// # Safety
  ///
  /// It's UB if `iter.map(|c| c.len_utf8()).sum()` > [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::StaticString;
  /// let string = unsafe { StaticString::<20>::from_chars_unchecked("My String".chars()) };
  /// assert_eq!(string.as_str(), "My String");
  ///
  /// // Undefined behavior, don't do it
  /// // let out_of_bounds = "000".repeat(400);
  /// // let undefined_behavior = unsafe { StaticString::<20>::from_chars_unchecked(out_of_bounds.chars()) };
  /// ```
  #[inline]
  pub unsafe fn from_chars_unchecked<I>(iter: I) -> Self
  where I: IntoIterator<Item = char> {
    let mut out = Self::default();
    for c in iter {
      out.push_unchecked(c)
    }
    out
  }

  /// Creates new `StaticString` from byte slice, returning [`Utf8`] on invalid utf-8 data or
  /// [`OutOfBounds`] if bigger than [`capacity`]
  ///
  /// [`Utf8`]: ./error/enum.StaticStringError.html#variant.Utf8
  /// [`OutOfBounds`]: ./error/enum.StaticStringError.html#variant.OutOfBounds
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let string = StaticString::<20>::try_from_utf8("My String")?;
  /// assert_eq!(string.as_str(), "My String");
  ///
  /// let invalid_utf8 = [0, 159, 146, 150];
  /// assert!(StaticString::<20>::try_from_utf8(invalid_utf8).unwrap_err().is_utf8());
  ///
  /// let out_of_bounds = "0000".repeat(400);
  /// assert!(StaticString::<20>::try_from_utf8(out_of_bounds.as_bytes()).unwrap_err().is_out_of_bounds());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_from_utf8<B>(slice: B) -> Result<Self, StaticStringError>
  where B: AsRef<[u8]> {
    Ok(Self::try_from_str(from_utf8(slice.as_ref())?)?)
  }

  /// Creates new `StaticString` from byte slice, returning [`Utf8`] on invalid utf-8 data,
  /// truncating if bigger than [`capacity`].
  ///
  /// [`Utf8`]: ./error/struct.Utf8.html
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let string = StaticString::<20>::from_utf8("My String")?;
  /// assert_eq!(string.as_str(), "My String");
  ///
  /// let invalid_utf8 = [0, 159, 146, 150];
  /// assert!(StaticString::<20>::from_utf8(invalid_utf8).unwrap_err().is_utf8());
  ///
  /// let out_of_bounds = "0".repeat(300);
  /// assert_eq!(StaticString::<20>::from_utf8(out_of_bounds.as_bytes())?.as_str(),
  ///            "0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn from_utf8<B>(slice: B) -> Result<Self, StaticStringError>
  where B: AsRef<[u8]> {
    Ok(Self::from_str_truncate(from_utf8(slice.as_ref())?))
  }

  /// Creates new `StaticString` from byte slice assuming it's utf-8 and of a appropriate size.
  ///
  /// # Safety
  ///
  /// It's UB if `slice` is not a valid utf-8 string or `slice.len()` > [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::StaticString;
  /// let string = unsafe { StaticString::<20>::from_utf8_unchecked("My String") };
  /// assert_eq!(string.as_str(), "My String");
  ///
  /// // Undefined behavior, don't do it
  /// // let out_of_bounds = "0".repeat(300);
  /// // let ub = unsafe { StaticString::<20>::from_utf8_unchecked(out_of_bounds)) };
  /// ```
  #[inline]
  pub unsafe fn from_utf8_unchecked<B>(slice: B) -> Self
  where B: AsRef<[u8]> {
    debug_assert!(from_utf8(slice.as_ref()).is_ok());
    Self::from_str_unchecked(from_utf8_unchecked(slice.as_ref()))
  }

  /// Creates new `StaticString` from `u16` slice, returning [`Utf16`] on invalid utf-16 data or
  /// [`OutOfBounds`] if bigger than [`capacity`]
  ///
  /// [`Utf16`]: ./error/enum.StaticStringError.html#variant.Utf16
  /// [`OutOfBounds`]: ./error/enum.StaticStringError.html#variant.OutOfBounds
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
  /// let string = StaticString::<20>::try_from_utf16(music)?;
  /// assert_eq!(string.as_str(), "𝄞music");
  ///
  /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
  /// assert!(StaticString::<20>::try_from_utf16(invalid_utf16).unwrap_err().is_utf16());
  ///
  /// let out_of_bounds: Vec<_> = (0..300).map(|_| 0).collect();
  /// assert!(StaticString::<20>::try_from_utf16(out_of_bounds).unwrap_err().is_out_of_bounds());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_from_utf16<B>(slice: B) -> Result<Self, StaticStringError>
  where B: AsRef<[u16]> {
    let mut out = Self::default();
    for c in decode_utf16(slice.as_ref().iter().cloned()) {
      out.try_push(c?)?;
    }
    Ok(out)
  }

  /// Creates new `StaticString` from `u16` slice, returning [`Utf16`] on invalid utf-16 data,
  /// truncating if bigger than [`capacity`].
  ///
  /// [`Utf16`]: ./error/struct.Utf16.html
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
  /// let string = StaticString::<20>::from_utf16(music)?;
  /// assert_eq!(string.as_str(), "𝄞music");
  ///
  /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
  /// assert!(StaticString::<20>::from_utf16(invalid_utf16).unwrap_err().is_utf16());
  ///
  /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
  /// assert_eq!(StaticString::<20>::from_utf16(out_of_bounds)?.as_str(),
  ///            "\0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn from_utf16<B>(slice: B) -> Result<Self, StaticStringError>
  where B: AsRef<[u16]> {
    let mut out = Self::default();
    for c in decode_utf16(slice.as_ref().iter().cloned()) {
      if out.try_push(c?).is_err() {
        break;
      }
    }
    Ok(out)
  }

  /// Creates new `StaticString` from `u16` slice, replacing invalid utf-16 data with
  /// `REPLACEMENT_CHARACTER` (\u{FFFD}) and truncating size if bigger than [`capacity`]
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
  /// let string = StaticString::<20>::from_utf16_lossy(music);
  /// assert_eq!(string.as_str(), "𝄞music");
  ///
  /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
  /// assert_eq!(StaticString::<20>::from_utf16_lossy(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
  ///
  /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
  /// assert_eq!(StaticString::<20>::from_utf16_lossy(&out_of_bounds).as_str(),
  ///            "\0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn from_utf16_lossy<B>(slice: B) -> Self
  where B: AsRef<[u16]> {
    let mut out = Self::default();
    for c in decode_utf16(slice.as_ref().iter().cloned()) {
      if out.try_push(c.unwrap_or(REPLACEMENT_CHARACTER)).is_err() {
        break;
      }
    }
    out
  }

  /// Extracts a string slice containing the entire `StaticString`
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let s = StaticString::<20>::try_from_str("My String")?;
  /// assert_eq!(s.as_str(), "My String");
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub const fn as_str(&self) -> &str {
    unsafe { &*(self.as_bytes() as *const [u8] as *const str) }
  }

  /// Extracts a mutable string slice containing the entire `StaticString`
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// assert_eq!(s.as_mut_str(), "My String");
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn as_mut_str(&mut self) -> &mut str {
    unsafe { &mut *(self.as_mut_bytes() as *mut [u8] as *mut str) }
  }

  /// Extracts a byte slice containing the entire `StaticString`
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let s = StaticString::<20>::try_from_str("My String")?;
  /// assert_eq!(s.as_bytes(), "My String".as_bytes());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub const fn as_bytes(&self) -> &[u8] {
    self.vec.as_slice()
  }

  /// Extracts a mutable string slice containing the entire `StaticString`
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// assert_eq!(unsafe { s.as_mut_bytes() }, "My String".as_bytes());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub const unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
    self.vec.as_mut_slice()
  }

  /// Returns maximum string capacity, defined at compile time, it will never change
  ///
  /// ```rust
  /// # use staticvec::StaticString;
  /// assert_eq!(StaticString::<32>::new().capacity(), 32);
  /// ```
  #[inline]
  pub const fn capacity(&self) -> usize {
    self.vec.capacity()
  }

  /// Pushes string slice to the end of the `StaticString` if total size is lower or equal to
  /// [`capacity`], otherwise returns an error.
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<300>::try_from_str("My String")?;
  /// s.try_push_str(" My other String")?;
  /// assert_eq!(s.as_str(), "My String My other String");
  ///
  /// assert!(s.try_push_str("0".repeat(300)).is_err());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_push_str<S>(&mut self, string: S) -> Result<(), StaticStringError>
  where S: AsRef<str> {
    let new_end = string.as_ref().len().saturating_add(self.len());
    is_inside_boundary(new_end, self.capacity())?;
    unsafe { self.push_str_unchecked(string) };
    Ok(())
  }

  /// Pushes string slice to the end of the `StaticString` truncating total size if bigger than
  /// [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<300>::try_from_str("My String")?;
  /// s.push_str(" My other String");
  /// assert_eq!(s.as_str(), "My String My other String");
  ///
  /// let mut s = StaticString::<20>::default();
  /// s.push_str("0".repeat(21));
  /// assert_eq!(s.as_str(), "0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn push_str<S>(&mut self, string: S)
  where S: AsRef<str> {
    let size = self.capacity().saturating_sub(self.len());
    unsafe { self.push_str_unchecked(truncate_str(string.as_ref(), size)) }
  }

  /// Pushes string slice to the end of the `StaticString` assuming total size is appropriate.
  ///
  /// # Safety
  ///
  /// It's UB if `self.len() + string.len()` > [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<300>::try_from_str("My String")?;
  /// unsafe { s.push_str_unchecked(" My other String") };
  /// assert_eq!(s.as_str(), "My String My other String");
  ///
  /// // Undefined behavior, don't do it
  /// // let mut undefined_behavior = StaticString::<20>::default();
  /// // undefined_behavior.push_str_unchecked("0".repeat(21));
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub unsafe fn push_str_unchecked<S>(&mut self, string: S)
  where S: AsRef<str> {
    let (s, len) = (string.as_ref(), string.as_ref().len());
    debug_assert!(len.saturating_add(self.len()) <= self.capacity());
    let dest = self.vec.as_mut_ptr().add(self.len());
    s.as_ptr().copy_to_nonoverlapping(dest, len);
    self.vec.set_len(self.len().saturating_add(len));
  }

  /// Inserts character to the end of the `StaticString` erroring if total size if bigger than
  /// [`capacity`].
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// s.try_push('!')?;
  /// assert_eq!(s.as_str(), "My String!");
  ///
  /// let mut s = StaticString::<20>::try_from_str(&"0".repeat(20))?;
  /// assert!(s.try_push('!').is_err());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_push(&mut self, character: char) -> Result<(), StaticStringError> {
    let new_end = character.len_utf8().saturating_add(self.len());
    is_inside_boundary(new_end, self.capacity())?;
    unsafe { self.push_unchecked(character) };
    Ok(())
  }

  /// Inserts character to the end of the `StaticString` assuming length is appropriate
  ///
  /// # Safety
  ///
  /// It's UB if `self.len() + character.len_utf8()` > [`capacity`]
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// unsafe { s.push_unchecked('!') };
  /// assert_eq!(s.as_str(), "My String!");
  ///
  /// // s = StaticString::<20>::try_from_str(&"0".repeat(20))?;
  /// // Undefined behavior, don't do it
  /// // s.push_unchecked('!');
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub unsafe fn push_unchecked(&mut self, ch: char) {
    encode_char_utf8_unchecked(self, ch, self.len());
  }

  /// Truncates `StaticString` to specified size (if smaller than current size and a valid utf-8
  /// char index).
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// s.truncate(5)?;
  /// assert_eq!(s.as_str(), "My St");
  ///
  /// // Does nothing
  /// s.truncate(6)?;
  /// assert_eq!(s.as_str(), "My St");
  ///
  /// // Index is not at a valid char
  /// let mut s = StaticString::<20>::try_from_str("🤔")?;
  /// assert!(s.truncate(1).is_err());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn truncate(&mut self, size: usize) -> Result<(), StaticStringError> {
    let len = min(self.len(), size);
    is_char_boundary(self, len).map(|()| unsafe { self.vec.set_len(len) })
  }

  /// Removes last character from `StaticString`, if any.
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("A🤔")?;
  /// assert_eq!(s.pop(), Some('🤔'));
  /// assert_eq!(s.pop(), Some('A'));
  /// assert_eq!(s.pop(), None);
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn pop(&mut self) -> Option<char> {
    self.as_str().chars().last().map(|ch| {
      unsafe { self.vec.set_len(self.len().saturating_sub(ch.len_utf8())) };
      ch
    })
  }

  /// Removes spaces from the beggining and end of the string
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut string = StaticString::<300>::try_from_str("   to be trimmed     ")?;
  /// string.trim();
  /// assert_eq!(string.as_str(), "to be trimmed");
  ///
  /// let mut string = StaticString::<20>::try_from_str("   🤔")?;
  /// string.trim();
  /// assert_eq!(string.as_str(), "🤔");
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn trim(&mut self) {
    let is_whitespace = |s: &[u8], index: usize| {
      debug_assert!(index < s.len());
      unsafe { s.get_unchecked(index) == &b' ' }
    };
    let (mut start, mut end, mut leave) = (0_usize, self.len(), 0_usize);
    while start < end && leave < 2 {
      leave = 0;

      if is_whitespace(self.as_bytes(), start) {
        start = start.saturating_add(1);
        if start >= end {
          continue;
        };
      } else {
        leave = leave.saturating_add(1);
      }

      if start < end && is_whitespace(self.as_bytes(), end.saturating_sub(1)) {
        end = end.saturating_sub(1);
      } else {
        leave = leave.saturating_add(1);
      }
    }

    unsafe { shift_left_unchecked(self, start, 0usize) };
    unsafe { self.vec.set_len(end.saturating_sub(start)) };
  }

  /// Removes specified char from `StaticString`
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// assert!(s.remove("ABCD🤔".len()).unwrap_err().is_out_of_bounds());
  /// assert!(s.remove(10).unwrap_err().is_out_of_bounds());
  /// assert!(s.remove(6).unwrap_err().isnt_char_boundary());
  /// assert_eq!(s.remove(0), Ok('A'));
  /// assert_eq!(s.as_str(), "BCD🤔");
  /// assert_eq!(s.remove(2), Ok('D'));
  /// assert_eq!(s.as_str(), "BC🤔");
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn remove(&mut self, idx: usize) -> Result<char, StaticStringError> {
    is_inside_boundary(idx, self.len().saturating_sub(1))?;
    is_char_boundary(self, idx)?;
    debug_assert!(idx < self.len() && self.as_str().is_char_boundary(idx));
    let ch = unsafe { self.as_str().get_unchecked(idx..).chars().next() };
    let ch = ch.unwrap_or_else(|| unsafe { never("Missing char") });
    unsafe { shift_left_unchecked(self, idx.saturating_add(ch.len_utf8()), idx) };
    unsafe { self.vec.set_len(self.len().saturating_sub(ch.len_utf8())) };
    Ok(ch)
  }

  /// Retains only the characters specified by the predicate.
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// s.retain(|c| c != '🤔');
  /// assert_eq!(s.as_str(), "ABCD");
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn retain<F: FnMut(char) -> bool>(&mut self, mut f: F) {
    // Not the most efficient solution, we could shift left during batch mismatch
    *self = unsafe { Self::from_chars_unchecked(self.as_str().chars().filter(|c| f(*c))) };
  }

  /// Inserts character at specified index, returning error if total length is bigger than
  /// [`capacity`].
  ///
  /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`Utf8`] if `idx` is not a char position
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  /// [`OutOfBounds`]: ./error/enum.StaticStringError.html#variant.OutOfBounds
  /// [`Utf8`]: ./error/enum.StaticStringError.html#variant.Utf8
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// s.try_insert(1, 'E')?;
  /// s.try_insert(2, 'F')?;
  /// assert_eq!(s.as_str(), "AEFBCD🤔");
  /// assert!(s.try_insert(20, 'C').unwrap_err().is_out_of_bounds());
  /// assert!(s.try_insert(8, 'D').unwrap_err().isnt_char_boundary());
  ///
  /// let mut s = StaticString::<20>::try_from_str(&"0".repeat(20))?;
  /// assert!(s.try_insert(0, 'C').unwrap_err().is_out_of_bounds());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_insert(&mut self, idx: usize, ch: char) -> Result<(), StaticStringError> {
    is_inside_boundary(idx, self.len())?;
    let new_end = ch.len_utf8().saturating_add(self.len());
    is_inside_boundary(new_end, self.capacity())?;
    is_char_boundary(self, idx)?;
    unsafe { self.insert_unchecked(idx, ch) };
    Ok(())
  }

  /// Inserts character at specified index assuming length is appropriate
  ///
  /// # Safety
  ///
  /// It's UB if `idx` does not lie on a utf-8 `char` boundary
  ///
  /// It's UB if `self.len() + character.len_utf8()` > [`capacity`]
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// unsafe { s.insert_unchecked(1, 'A') };
  /// unsafe { s.insert_unchecked(1, 'B') };
  /// assert_eq!(s.as_str(), "ABABCD🤔");
  ///
  /// // Undefined behavior, don't do it
  /// // s.insert(20, 'C');
  /// // s.insert(8, 'D');
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub unsafe fn insert_unchecked(&mut self, idx: usize, ch: char) {
    let clen = ch.len_utf8();
    shift_right_unchecked(self, idx, idx.saturating_add(clen));
    encode_char_utf8_unchecked(self, ch, idx);
  }

  /// Inserts string slice at specified index, returning error if total length is bigger than
  /// [`capacity`].
  ///
  /// Returns [`OutOfBounds`] if `idx` is out of bounds
  /// Returns [`Utf8`] if `idx` is not a char position
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  /// [`OutOfBounds`]: ./error/enum.StaticStringError.html#variant.OutOfBounds
  /// [`Utf8`]: ./error/enum.StaticStringError.html#variant.Utf8
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// s.try_insert_str(1, "AB")?;
  /// s.try_insert_str(1, "BC")?;
  /// assert!(s.try_insert_str(1, "0".repeat(20)).unwrap_err().is_out_of_bounds());
  /// assert_eq!(s.as_str(), "ABCABBCD🤔");
  /// assert!(s.try_insert_str(20, "C").unwrap_err().is_out_of_bounds());
  /// assert!(s.try_insert_str(10, "D").unwrap_err().isnt_char_boundary());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_insert_str<S>(&mut self, idx: usize, s: S) -> Result<(), StaticStringError>
  where S: AsRef<str> {
    is_inside_boundary(idx, self.len())?;
    let new_end = s.as_ref().len().saturating_add(self.len());
    is_inside_boundary(new_end, self.capacity())?;
    is_char_boundary(self, idx)?;
    unsafe { self.insert_str_unchecked(idx, s.as_ref()) };
    Ok(())
  }

  /// Inserts string slice at specified index, truncating size if bigger than [`capacity`].
  ///
  /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`Utf8`] if `idx` is not a char position
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  /// [`OutOfBounds`]: ./error/enum.StaticStringError.html#variant.OutOfBounds
  /// [`Utf8`]: ./error/enum.StaticStringError.html#variant.Utf8
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// s.insert_str(1, "AB")?;
  /// s.insert_str(1, "BC")?;
  /// assert_eq!(s.as_str(), "ABCABBCD🤔");
  ///
  /// assert!(s.insert_str(20, "C").unwrap_err().is_out_of_bounds());
  /// assert!(s.insert_str(10, "D").unwrap_err().isnt_char_boundary());
  ///
  /// s.clear();
  /// s.insert_str(0, "0".repeat(30))?;
  /// assert_eq!(s.as_str(), "0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn insert_str<S>(&mut self, idx: usize, string: S) -> Result<(), StaticStringError>
  where S: AsRef<str> {
    is_inside_boundary(idx, self.len())?;
    is_char_boundary(self, idx)?;
    let size = self.capacity().saturating_sub(self.len());
    unsafe { self.insert_str_unchecked(idx, truncate_str(string.as_ref(), size)) };
    Ok(())
  }

  /// Inserts string slice at specified index, assuming total length is appropriate.
  ///
  /// # Safety
  ///
  /// It's UB if `idx` does not lie on a utf-8 `char` boundary
  ///
  /// It's UB if `self.len() + string.len()` > [`capacity`]
  ///
  /// [`capacity`]: ./struct.StaticString.html#method.capacity
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// unsafe { s.insert_str_unchecked(1, "AB") };
  /// unsafe { s.insert_str_unchecked(1, "BC") };
  /// assert_eq!(s.as_str(), "ABCABBCD🤔");
  ///
  /// // Undefined behavior, don't do it
  /// // unsafe { s.insert_str_unchecked(20, "C") };
  /// // unsafe { s.insert_str_unchecked(10, "D") };
  /// // unsafe { s.insert_str_unchecked(1, "0".repeat(20)) };
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub unsafe fn insert_str_unchecked<S>(&mut self, idx: usize, string: S)
  where S: AsRef<str> {
    let (s, slen) = (string.as_ref(), string.as_ref().len());
    let ptr = s.as_ptr();
    debug_assert!(self.len().saturating_add(slen) <= self.capacity());
    debug_assert!(idx <= self.len());
    debug_assert!(self.as_str().is_char_boundary(idx));

    shift_right_unchecked(self, idx, idx.saturating_add(slen));
    let dest = self.vec.as_mut_ptr().add(idx);
    ptr.copy_to_nonoverlapping(dest, slen);
    self.vec.set_len(self.len().saturating_add(slen));
  }

  /// Returns `StaticString` length.
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD")?;
  /// assert_eq!(s.len(), 4);
  /// s.try_push('🤔')?;
  /// // Emojis use 4 bytes (this is the default rust behavior, length of usize)
  /// assert_eq!(s.len(), 8);
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub const fn len(&self) -> usize {
    self.vec.len()
  }

  /// Checks if `StaticString` is empty.
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD")?;
  /// assert!(!s.is_empty());
  /// s.clear();
  /// assert!(s.is_empty());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub const fn is_empty(&self) -> bool {
    self.vec.is_empty()
  }

  /// Splits `StaticString` in two if `at` is smaller than `self.len()`.
  ///
  /// Returns [`Utf8`] if `at` does not lie at a valid utf-8 char boundary and [`OutOfBounds`] if
  /// it's out of bounds
  ///
  /// [`OutOfBounds`]: ./error/enum.StaticStringError.html#variant.OutOfBounds
  /// [`Utf8`]: ./error/enum.StaticStringError.html#variant.Utf8
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("AB🤔CD")?;
  /// assert_eq!(s.split_off(6)?.as_str(), "CD");
  /// assert_eq!(s.as_str(), "AB🤔");
  /// assert!(s.split_off(20).unwrap_err().is_out_of_bounds());
  /// assert!(s.split_off(4).unwrap_err().isnt_char_boundary());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn split_off(&mut self, at: usize) -> Result<Self, StaticStringError> {
    is_inside_boundary(at, self.len())?;
    is_char_boundary(self, at)?;
    debug_assert!(at <= self.len() && self.as_str().is_char_boundary(at));
    let new = unsafe { Self::from_utf8_unchecked(self.as_str().get_unchecked(at..)) };
    unsafe { self.vec.set_len(at) };
    Ok(new)
  }

  /// Empties `StaticString`
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD")?;
  /// assert!(!s.is_empty());
  /// s.clear();
  /// assert!(s.is_empty());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn clear(&mut self) {
    unsafe { self.vec.set_len(0) };
  }

  /// Removes the specified range of the `StaticString`, and replaces it with the given string. The
  /// given string doesn't need to have the same length as the range.
  ///
  /// ```rust
  /// # use staticvec::{StaticString, StaticStringError};
  /// # fn main() -> Result<(), StaticStringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// s.replace_range(2..4, "EFGHI")?;
  /// assert_eq!(s.as_str(), "ABEFGHI🤔");
  ///
  /// assert!(s.replace_range(9.., "J").unwrap_err().isnt_char_boundary());
  /// assert!(s.replace_range(..90, "K").unwrap_err().is_out_of_bounds());
  /// assert!(s.replace_range(0..1, "0".repeat(20)).unwrap_err().is_out_of_bounds());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn replace_range<S, R>(&mut self, r: R, with: S) -> Result<(), StaticStringError>
  where
    S: AsRef<str>,
    R: RangeBounds<usize>, {
    let replace_with = with.as_ref();
    let start = match r.start_bound() {
      Bound::Included(t) => *t,
      Bound::Excluded(t) => t.saturating_add(1),
      Bound::Unbounded => 0,
    };
    let end = match r.end_bound() {
      Bound::Included(t) => t.saturating_add(1),
      Bound::Excluded(t) => *t,
      Bound::Unbounded => self.len(),
    };

    let len = replace_with.len();

    is_inside_boundary(start, end)?;
    is_inside_boundary(end, self.len())?;
    let replaced = end.saturating_sub(start);
    is_inside_boundary(replaced.saturating_add(len), self.capacity())?;
    is_char_boundary(self, start)?;
    is_char_boundary(self, end)?;

    debug_assert!(start <= end && end <= self.len());
    debug_assert!(len.saturating_sub(end).saturating_add(start) <= self.capacity());
    debug_assert!(self.as_str().is_char_boundary(start));
    debug_assert!(self.as_str().is_char_boundary(end));

    if start.saturating_add(len) > end {
      unsafe { shift_right_unchecked(self, end, start.saturating_add(len)) };
    } else {
      unsafe { shift_left_unchecked(self, end, start.saturating_add(len)) };
    }

    let grow = len.saturating_sub(replaced);
    unsafe { self.vec.set_len(self.len().saturating_add(grow)) };
    let ptr = replace_with.as_ptr();
    let dest = unsafe { self.vec.as_mut_ptr().add(start) };
    unsafe { ptr.copy_to_nonoverlapping(dest, len) };
    Ok(())
  }
}
