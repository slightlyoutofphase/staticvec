use core::char::{decode_utf16, REPLACEMENT_CHARACTER};
use core::ops::{Bound, RangeBounds};
use core::str::{self, from_utf8, from_utf8_unchecked};

pub use self::string_errors::StringError;
use self::string_utils::{
  encode_char_utf8_unchecked, is_char_boundary, is_inside_boundary, never, shift_left_unchecked,
  shift_right_unchecked, truncate_str,
};
use crate::errors::CapacityError;
use crate::StaticVec;

#[cfg(all(feature = "std", rustdoc))]
use alloc::string::String;

mod string_errors;
mod string_trait_impls;
#[doc(hidden)]
pub mod string_utils;

/// A fixed-capacity [`String`](alloc::string::String)-like struct built around an instance of
/// `StaticVec<u8, N>`.
///
/// ## Examples
///
/// ```
/// use staticvec::{StaticString, StringError};
///
/// #[derive(Debug)]
/// pub struct User {
///   pub username: StaticString<20>,
///   pub role: StaticString<5>,
/// }
///
/// fn main() -> Result<(), StringError> {
///   let user = User {
///     username: StaticString::try_from_str("user")?,
///     role: StaticString::try_from_str("admin")?,
///   };
///   println!("{:?}", user);
///   Ok(())
/// }
/// ```
pub struct StaticString<const N: usize> {
  pub(crate) vec: StaticVec<u8, N>,
}

impl<const N: usize> StaticString<N> {
  /// Returns a new StaticString instance.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let string = StaticString::<20>::new();
  /// assert!(string.is_empty());
  /// ```
  #[inline(always)]
  pub const fn new() -> Self {
    Self {
      vec: StaticVec::new(),
    }
  }

  /// An internal "const constructor" helper function that exists to support the `staticstring!`
  /// macro. This is only used in one place at the moment, where the input `StaticVec` is known to
  /// have been built from a valid-UTF8 `&'static str` literal. Since this has to be accessible from
  /// the macro when invoked from external crates, and so can't be `pub(crate)`, we hide it from the
  /// docs and give it a two-underscore prefix as the next best thing.
  #[doc(hidden)]
  #[inline(always)]
  pub const unsafe fn __new_from_staticvec(vec: StaticVec<u8, N>) -> Self {
    Self { vec }
  }

  /// Creates a new StaticString instance from `string`, without doing any checking to ensure that
  /// the length of `string` does not exceed the resulting StaticString's declared capacity.
  ///
  /// # Safety
  ///
  /// The length of `string` must not exceed the declared capacity of the StaticString being
  /// created, as this would result in writing to an out-of-bounds memory region.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let string = unsafe { StaticString::<20>::from_str_unchecked("My String") };
  /// assert_eq!(string, "My String");
  /// ```
  #[inline(always)]
  pub unsafe fn from_str_unchecked(string: &str) -> Self {
    let mut res = Self::new();
    res.push_str_unchecked(string);
    res
  }

  /// Creates a new StaticString from `string`, truncating `string` as necessary if it has
  /// a length greater than the StaticString's declared capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let string = StaticString::<20>::from_str("My String");
  /// assert_eq!(string, "My String");
  /// let truncate = "0".repeat(21);
  /// let truncated = "0".repeat(20);
  /// let string = StaticString::<20>::from_str(&truncate);
  /// assert_eq!(string, truncated.as_str());
  /// ```
  #[allow(clippy::should_implement_trait)]
  #[inline(always)]
  pub fn from_str<S: AsRef<str>>(string: S) -> Self {
    let mut res = Self::new();
    let string_ref = string.as_ref();
    unsafe {
      match string_ref.len() <= N {
        false => res.push_str_unchecked(truncate_str(string_ref, N)),
        true => res.push_str_unchecked(string_ref),
      }
    }
    res
  }

  /// Creates a new StaticString from `string` if the length of `string` is less than or equal
  /// to the StaticString's declared capacity, or returns a
  /// [`CapacityError`](crate::errors::CapacityError) otherwise.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let string = StaticString::<20>::from("My String");
  /// assert_eq!(string.as_str(), "My String");
  /// assert_eq!(StaticString::<20>::try_from_str("").unwrap().as_str(), "");
  /// let out_of_bounds = "0".repeat(21);
  /// assert!(StaticString::<20>::try_from_str(out_of_bounds).is_err());
  /// ```
  #[inline(always)]
  pub fn try_from_str<S: AsRef<str>>(string: S) -> Result<Self, CapacityError<N>> {
    let mut res = Self::new();
    res.try_push_str(string)?;
    Ok(res)
  }

  /// Creates a new StaticString from the contents of an iterator, returning immediately if and when
  /// the StaticString reaches maximum capacity regardless of whether or not the iterator still has
  /// more items to yield.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let string = StaticString::<300>::from_iterator(&["My String", " Other String"][..]);
  /// assert_eq!(string.as_str(), "My String Other String");
  /// let out_of_bounds = (0..400).map(|_| "000");
  /// let truncated = "0".repeat(18);
  /// let truncate = StaticString::<20>::from_iterator(out_of_bounds);
  /// assert_eq!(truncate.as_str(), truncated.as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn from_iterator<U: AsRef<str>, I: IntoIterator<Item = U>>(iter: I) -> Self {
    let mut res = Self::new();
    for s in iter {
      let s_ref = s.as_ref();
      match res.remaining_capacity() < s_ref.len() {
        false => unsafe { res.push_str_unchecked(s_ref) },
        true => break,
      }
    }
    res
  }

  /// Creates a new StaticString from the contents of an iterator if the iterator has a length less
  /// than or equal to the StaticString's declared capacity, or returns a
  /// [`CapacityError`](crate::errors::CapacityError) otherwise.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let string = StaticString::<300>::try_from_iterator(
  ///   &["My String", " My Other String"][..]
  /// ).unwrap();
  /// assert_eq!(string.as_str(), "My String My Other String");
  /// let out_of_bounds = (0..100).map(|_| "000");
  /// assert!(StaticString::<20>::try_from_iterator(out_of_bounds).is_err());
  /// ```
  #[inline]
  pub fn try_from_iterator<U: AsRef<str>, I: IntoIterator<Item = U>>(
    iter: I,
  ) -> Result<Self, CapacityError<N>> {
    let mut res = Self::new();
    for s in iter {
      res.try_push_str(s)?;
    }
    Ok(res)
  }

  /// Creates a new StaticString from the contents of a `char` iterator, returning immediately if
  /// and when the StaticString reaches maximum capacity regardless of whether or not the iterator
  /// still has more items to yield.
  ///
  /// ```
  /// # use staticvec::StaticString;
  /// let string = StaticString::<20>::from_chars("My String".chars());
  /// assert_eq!(string, "My String");
  /// let out_of_bounds = "0".repeat(21);
  /// let truncated = "0".repeat(20);
  /// let truncate = StaticString::<20>::from_chars(out_of_bounds.chars());
  /// assert_eq!(truncate.as_str(), truncated.as_str());
  /// ```
  #[inline]
  pub fn from_chars<I: IntoIterator<Item = char>>(iter: I) -> Self {
    let mut res = Self::new();
    for c in iter {
      match res.remaining_capacity() < c.len_utf8() {
        false => unsafe { res.push_unchecked(c) },
        true => break,
      }
    }
    res
  }

  /// Creates a new StaticString from the contents of a `char` iterator if the iterator has a length
  /// less than or equal to the StaticString's declared capacity, or returns
  /// [`StringError::OutOfBounds`] otherwise.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let string = StaticString::<20>::try_from_chars("My String".chars())?;
  /// assert_eq!(string.as_str(), "My String");
  /// let out_of_bounds = "0".repeat(21);
  /// assert!(StaticString::<20>::try_from_chars(out_of_bounds.chars()).is_err());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_from_chars<I: IntoIterator<Item = char>>(iter: I) -> Result<Self, StringError> {
    let mut res = Self::new();
    for c in iter {
      res.try_push(c)?;
    }
    Ok(res)
  }

  /// Creates a new StaticString instance from the provided byte slice, without doing any checking
  /// to ensure that the slice contains valid UTF-8 data and has a length less than or equal to
  /// the declared capacity of the StaticString.
  ///
  /// # Safety
  ///
  /// The length of the slice must not exceed the declared capacity of the StaticString being
  /// created, as this would result in writing to an out-of-bounds memory region.
  ///
  /// The slice must also contain strictly valid UTF-8 data, as if it does not, various assumptions
  /// made in the internal implementation of StaticString will be silently invalidated, almost
  /// certainly eventually resulting in undefined behavior.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let string = unsafe { StaticString::<20>::from_utf8_unchecked("My String") };
  /// assert_eq!(string, "My String");
  /// // Undefined behavior, don't do it:
  /// // let out_of_bounds = "0".repeat(300);
  /// // let ub = unsafe { StaticString::<20>::from_utf8_unchecked(out_of_bounds)) };
  /// ```
  #[inline(always)]
  pub unsafe fn from_utf8_unchecked<B: AsRef<[u8]>>(slice: B) -> Self {
    debug_assert!(from_utf8(slice.as_ref()).is_ok());
    Self::from_str_unchecked(from_utf8_unchecked(slice.as_ref()))
  }

  /// Creates a new StaticString instance from the provided byte slice, returning
  /// [`StringError::Utf8`] on invalid UTF-8 data, and truncating the input slice as necessary if
  /// it has a length greater than the declared capacity of the StaticString being created.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let string = StaticString::<20>::from_utf8("My String")?;
  /// assert_eq!(string, "My String");
  /// let invalid_utf8 = [0, 159, 146, 150];
  /// assert!(StaticString::<20>::from_utf8(invalid_utf8).unwrap_err().is_utf8());
  /// let out_of_bounds = "0".repeat(300);
  /// assert_eq!(StaticString::<20>::from_utf8(out_of_bounds.as_bytes())?.as_str(), "0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub fn from_utf8<B: AsRef<[u8]>>(slice: B) -> Result<Self, StringError> {
    Ok(Self::from_str(from_utf8(slice.as_ref())?))
  }

  /// Creates a new StaticString from the provided byte slice, returning [`StringError::Utf8`] on
  /// invalid UTF-8 data or [`StringError::OutOfBounds`] if the slice has a length greater than
  /// the StaticString's declared capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let string = StaticString::<20>::try_from_utf8("My String")?;
  /// assert_eq!(string, "My String");
  /// let invalid_utf8 = [0, 159, 146, 150];
  /// assert!(StaticString::<20>::try_from_utf8(invalid_utf8).unwrap_err().is_utf8());
  /// let out_of_bounds = "0000".repeat(400);
  /// assert!(StaticString::<20>::try_from_utf8(out_of_bounds.as_bytes()).unwrap_err().is_out_of_bounds());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub fn try_from_utf8<B: AsRef<[u8]>>(slice: B) -> Result<Self, StringError> {
    Ok(Self::try_from_str(from_utf8(slice.as_ref())?)?)
  }

  /// Creates a new StaticString instance from the provided `u16` slice, replacing invalid UTF-16
  /// data with `REPLACEMENT_CHARACTER` (�), and truncating the input slice as necessary if
  /// it has a length greater than the declared capacity of the StaticString being created.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
  /// let string = StaticString::<20>::from_utf16_lossy(music);
  /// assert_eq!(string, "𝄞music");
  /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
  /// assert_eq!(StaticString::<20>::from_utf16_lossy(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
  /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
  /// assert_eq!(StaticString::<20>::from_utf16_lossy(&out_of_bounds).as_str(), "\0".repeat(20).as_str());
  /// ```
  #[inline(always)]
  pub fn from_utf16_lossy<B: AsRef<[u16]>>(slice: B) -> Self {
    let mut res = Self::new();
    for c in decode_utf16(slice.as_ref().iter().copied()) {
      if res.try_push(c.unwrap_or(REPLACEMENT_CHARACTER)).is_err() {
        break;
      }
    }
    res
  }

  /// Creates a new StaticString instance from the provided `u16` slice, returning
  /// [`StringError::Utf16`] on invalid UTF-16 data, and truncating the input slice as necessary if
  /// it has a length greater than the declared capacity of the StaticString being created.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
  /// let string = StaticString::<20>::from_utf16(music)?;
  /// assert_eq!(string.as_str(), "𝄞music");
  /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
  /// assert!(StaticString::<20>::from_utf16(invalid_utf16).unwrap_err().is_utf16());
  /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
  /// assert_eq!(StaticString::<20>::from_utf16(out_of_bounds)?.as_str(),
  ///            "\0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn from_utf16<B: AsRef<[u16]>>(slice: B) -> Result<Self, StringError> {
    let mut res = Self::new();
    for c in decode_utf16(slice.as_ref().iter().copied()) {
      if res.try_push(c?).is_err() {
        break;
      }
    }
    Ok(res)
  }

  /// Creates a new StaticString from the provided `u16` slice, returning [`StringError::Utf16`] on
  /// invalid UTF-16 data or [`StringError::OutOfBounds`] if the slice has a length greater than the
  /// declared capacity of the StaticString being created.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticVec, StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
  /// let string = StaticString::<20>::try_from_utf16(music)?;
  /// assert_eq!(string.as_str(), "𝄞music");
  /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
  /// assert!(StaticString::<20>::try_from_utf16(invalid_utf16).unwrap_err().is_utf16());
  /// let out_of_bounds: StaticVec<u16, 300> = (0..300).map(|_| 0).collect();
  /// assert!(StaticString::<20>::try_from_utf16(out_of_bounds).unwrap_err().is_out_of_bounds());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_from_utf16<B: AsRef<[u16]>>(slice: B) -> Result<Self, StringError> {
    let mut res = Self::new();
    for c in decode_utf16(slice.as_ref().iter().copied()) {
      res.try_push(c?)?;
    }
    Ok(res)
  }

  /// Extracts a `str` slice containing the entire contents of the StaticString.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let s = StaticString::<20>::from_str("My String");
  /// assert_eq!(s.as_str(), "My String");
  /// ```
  #[inline(always)]
  pub const fn as_str(&self) -> &str {
    unsafe { &*(self.as_bytes() as *const [u8] as *const str) }
  }

  /// Extracts a mutable `str` slice containing the entire contents of the StaticString.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from_str("My String");
  /// assert_eq!(s.as_mut_str(), "My String");
  /// ```
  #[inline(always)]
  pub const fn as_mut_str(&mut self) -> &mut str {
    unsafe { &mut *(self.as_mut_bytes() as *mut [u8] as *mut str) }
  }

  /// Extracts a `u8` slice containing the entire contents of the StaticString.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let s = StaticString::<20>::from_str("My String");
  /// assert_eq!(s.as_bytes(), "My String".as_bytes());
  /// ```
  #[inline(always)]
  pub const fn as_bytes(&self) -> &[u8] {
    self.vec.as_slice()
  }

  /// Returns the StaticString's internal instance of `StaticVec<u8, N>`.
  /// Note that using this function consumes the StaticString.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let s = StaticString::<5>::from("hello");
  /// let bytes = s.into_bytes();
  /// assert_eq!(&bytes[..], &[104, 101, 108, 108, 111][..]);
  /// ```
  #[inline(always)]
  pub fn into_bytes(self) -> StaticVec<u8, N> {
    self.vec
  }

  /// Extracts a mutable `u8` slice containing the entire contents of the StaticString.
  ///
  /// # Safety
  ///
  /// Care must be taken to ensure that the returned `u8` slice is not mutated in such a way that
  /// it no longer amounts to valid UTF-8.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// assert_eq!(unsafe { s.as_mut_bytes() }, "My String".as_bytes());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub const unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
    self.vec.as_mut_slice()
  }

  /// Returns a mutable reference to the StaticString's backing StaticVec.
  ///
  /// # Safety
  ///
  /// Care must be taken to ensure that the returned StaticVec reference is not mutated in such a
  /// way that it no longer contains valid UTF-8.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// assert_eq!(unsafe { s.as_mut_staticvec() }, "My String".as_bytes());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub const unsafe fn as_mut_staticvec(&mut self) -> &mut StaticVec<u8, N> {
    &mut self.vec
  }

  /// Returns the total capacity of the StaticString.
  /// This is always equivalent to the generic `N` parameter it was declared with,
  /// which determines the fixed size of the backing StaticVec instance.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// assert_eq!(StaticString::<32>::new().capacity(), 32);
  /// ```
  #[inline(always)]
  pub const fn capacity(&self) -> usize {
    self.vec.capacity()
  }

  /// Returns the remaining capacity (which is to say, `self.capacity() - self.len()`) of the
  /// StaticString.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// assert_eq!(StaticString::<32>::from("abcd").remaining_capacity(), 28);
  /// ```
  #[inline(always)]
  pub const fn remaining_capacity(&self) -> usize {
    self.vec.remaining_capacity()
  }

  /// Pushes `string` to the StaticString without doing any checking to ensure that `self.len() +
  /// string.len()` does not exceed the StaticString's total capacity.
  ///
  /// # Safety
  ///
  /// `self.len() + string.len()` must not exceed the total capacity of the StaticString
  /// instance, as this would result in writing to an out-of-bounds memory region.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString};
  /// let mut s = StaticString::<6>::from("foo");
  /// unsafe { s.push_str_unchecked("bar") };
  /// assert_eq!(s, "foobar");
  /// ```
  #[inline(always)]
  pub unsafe fn push_str_unchecked(&mut self, string: &str) {
    let string_length = string.len();
    debug_assert!(string_length <= self.remaining_capacity());
    let old_length = self.len();
    let dest = self.vec.mut_ptr_at_unchecked(old_length);
    string.as_ptr().copy_to_nonoverlapping(dest, string_length);
    self.vec.set_len(old_length + string_length);
  }

  /// Attempts to push `string` to the StaticString, panicking if it is the case that `self.len() +
  /// string.len()` exceeds the StaticString's total capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString};
  /// let mut s = StaticString::<6>::from("foo");
  /// s.push_str("bar");
  /// assert_eq!(s, "foobar");
  /// ```
  #[inline(always)]
  pub fn push_str<S: AsRef<str>>(&mut self, string: S) {
    let string_ref = string.as_ref();
    assert!(
      string_ref.len() <= self.remaining_capacity(),
      "Insufficient remaining capacity!"
    );
    unsafe { self.push_str_unchecked(string_ref) };
  }

  /// Attempts to push `string` to the StaticString. Truncates `string` as necessary (or simply does
  /// nothing at all) if it is the case that `self.len() + string.len()` exceeds the
  /// StaticString's total capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<300>::try_from_str("My String")?;
  /// s.push_str_truncating(" My other String");
  /// assert_eq!(s.as_str(), "My String My other String");
  /// let mut s = StaticString::<20>::new();
  /// s.push_str_truncating("0".repeat(21));
  /// assert_eq!(s.as_str(), "0".repeat(20).as_str());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub fn push_str_truncating<S: AsRef<str>>(&mut self, string: S) {
    unsafe { self.push_str_unchecked(truncate_str(string.as_ref(), self.remaining_capacity())) };
  }

  /// Pushes `string` to the StaticString if `self.len() + string.len()` does not exceed
  /// the StaticString's total capacity, or returns a
  /// [`CapacityError`](crate::errors::CapacityError) otherwise.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<300>::from("My String");
  /// s.try_push_str(" My other String").unwrap();
  /// assert_eq!(s.as_str(), "My String My other String");
  /// assert!(s.try_push_str("0".repeat(300)).is_err());
  /// ```
  #[inline(always)]
  pub fn try_push_str<S: AsRef<str>>(&mut self, string: S) -> Result<(), CapacityError<N>> {
    let string_ref = string.as_ref();
    match self.vec.remaining_capacity() < string_ref.len() {
      false => {
        unsafe { self.push_str_unchecked(string_ref) };
        Ok(())
      }
      true => Err(CapacityError {}),
    }
  }

  /// Appends the given char to the end of the StaticString without doing any checking to ensure
  /// that `self.len() + character.len_utf8()` does not exceed the total capacity of the
  /// StaticString instance.
  ///
  /// # Safety
  ///
  /// `self.len() + character.len_utf8()` must not exceed the total capacity of the StaticString
  /// instance, as this would result in writing to an out-of-bounds memory region.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// unsafe { s.push_unchecked('!') };
  /// assert_eq!(s.as_str(), "My String!");
  /// // Undefined behavior, don't do it:
  /// // s = StaticString::<20>::try_from_str(&"0".repeat(20))?;
  /// // s.push_unchecked('!');
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub unsafe fn push_unchecked(&mut self, character: char) {
    let char_length = character.len_utf8();
    match char_length {
      1 => self.vec.push_unchecked(character as u8),
      _ => {
        let old_length = self.len();
        character
          .encode_utf8(&mut [0; 4])
          .as_ptr()
          .copy_to_nonoverlapping(self.vec.mut_ptr_at_unchecked(old_length), char_length);
        self.vec.set_len(old_length + char_length);
      }
    };
  }

  /// Appends the given char to the end of the StaticString, panicking if the StaticString
  /// is already at maximum capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// let mut string = StaticString::<2>::new();
  /// string.push('a');
  /// string.push('b');
  /// assert_eq!(&string[..], "ab");
  /// ```
  #[inline(always)]
  pub fn push(&mut self, character: char) {
    assert!(
      character.len_utf8() <= self.remaining_capacity(),
      "Insufficient remaining capacity!"
    );
    unsafe { self.push_unchecked(character) };
  }

  /// Appends the given char to the end of the StaticString, returning [`StringError::OutOfBounds`]
  /// if the StaticString is already at maximum capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("My String")?;
  /// s.try_push('!')?;
  /// assert_eq!(s.as_str(), "My String!");
  /// let mut s = StaticString::<20>::try_from_str(&"0".repeat(20))?;
  /// assert!(s.try_push('!').is_err());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub fn try_push(&mut self, character: char) -> Result<(), StringError> {
    match self.remaining_capacity() < character.len_utf8() {
      false => {
        unsafe { self.push_unchecked(character) }
        Ok(())
      }
      true => Err(StringError::OutOfBounds),
    }
  }

  /// Truncates the StaticString to `new_len` if `new_len` is less than or equal to the
  /// StaticString's current length, or does nothing otherwise. Panics if `new_len` does not lie
  /// at a valid UTF-8 character boundary.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from("My String");
  /// s.truncate(5);
  /// assert_eq!(s, "My St");
  /// // Does nothing
  /// s.truncate(6);
  /// assert_eq!(s, "My St");
  /// // Would panic
  /// // let mut s2 = StaticString::<20>::from("🤔");
  /// // s2.truncate(1);
  /// ```
  #[inline(always)]
  pub fn truncate(&mut self, new_len: usize) {
    if new_len <= self.len() {
      assert!(self.as_str().is_char_boundary(new_len));
      unsafe { self.vec.set_len(new_len) };
    }
  }

  /// Returns the last character in the StaticString in `Some` if the StaticString's current length
  /// is greater than zero, or `None` otherwise.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("A🤔")?;
  /// assert_eq!(s.pop(), Some('🤔'));
  /// assert_eq!(s.pop(), Some('A'));
  /// assert_eq!(s.pop(), None);
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub fn pop(&mut self) -> Option<char> {
    self.as_str().chars().last().map(|character| {
      unsafe { self.vec.set_len(self.len() - character.len_utf8()) };
      character
    })
  }

  /// Removes all whitespace from the beginning and end of the StaticString, if any is present.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut string = StaticString::<300>::try_from_str("   to be trimmed     ")?;
  /// string.trim();
  /// assert_eq!(string.as_str(), "to be trimmed");
  /// let mut string = StaticString::<20>::try_from_str("   🤔")?;
  /// string.trim();
  /// assert_eq!(string.as_str(), "🤔");
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn trim(&mut self) {
    let is_whitespace = |bytes: &[u8], index: usize| {
      debug_assert!(index < bytes.len());
      unsafe { bytes.get_unchecked(index) == &b' ' }
    };
    let (mut start, mut end, mut leave) = (0_usize, self.len(), 0_usize);
    while start < end && leave < 2 {
      leave = 0;
      if is_whitespace(self.as_bytes(), start) {
        start += 1;
        if start >= end {
          continue;
        };
      } else {
        leave += 1;
      }
      if start < end && is_whitespace(self.as_bytes(), end - 1) {
        end -= 1;
      } else {
        leave += 1;
      }
    }
    unsafe {
      shift_left_unchecked(self, start, 0usize);
      self.vec.set_len(end - start)
    };
  }

  /// Removes the char at `index` from the StaticString if `index` is both less than `self.len()`
  /// and also lies at a valid UTF-8 character boundary, or panics otherwise.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from("ABCD🤔");
  /// assert_eq!(s.remove(0), 'A');
  /// assert!(s == "BCD🤔");
  /// assert_eq!(s.remove(2), 'D');
  /// assert!(s == "BC🤔");
  /// ```
  #[inline]
  pub fn remove(&mut self, index: usize) -> char {
    assert!(
      self.as_str().is_char_boundary(index),
      "Out of bounds or invalid character boundary!"
    );
    let old_length = self.len();
    let character = unsafe { self.as_str().get_unchecked(index..).chars().next() };
    let character = character.unwrap_or_else(|| unsafe { never("Missing char") });
    let char_length = character.len_utf8();
    unsafe {
      shift_left_unchecked(self, index + char_length, index);
      self.vec.set_len(old_length - char_length);
    }
    character
  }

  /// Removes all characters from the StaticString except for those specified by the predicate
  /// function.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from("ABCD🤔");
  /// s.retain(|c| c != '🤔');
  /// assert_eq!(s, "ABCD");
  /// ```
  #[inline(always)]
  pub fn retain<F: FnMut(char) -> bool>(&mut self, mut f: F) {
    // Having benched `retain` implemented both this way and exactly how `std::string::String`
    // does, this way is faster believe it or not.
    *self = Self::from_chars(self.as_str().chars().filter(|c| f(*c)));
  }

  /// Inserts `character` at `index`, shifting any values that exist in positions greater than
  /// `index` to the right.
  ///
  /// Does not do any checking to ensure that `character.len_utf8() + self.len()` does not exceed
  /// the total capacity of the StaticString or that `index` lies at a valid UTF-8 character
  /// boundary.
  ///
  /// # Safety
  ///
  /// The length of the StaticString prior to calling this function must be less than its total
  /// capacity, as if this in not the case it will result in writing to an out-of-bounds memory
  /// region.
  ///
  /// `Index` must also lie at a valid UTF-8 character boundary, as if it does not, various
  /// assumptions made in the internal implementation of StaticString will be silently
  /// invalidated, almost certainly eventually resulting in undefined behavior.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// unsafe { s.insert_unchecked(1, 'A') };
  /// unsafe { s.insert_unchecked(1, 'B') };
  /// assert_eq!(s.as_str(), "ABABCD🤔");
  /// // Undefined behavior, don't do it:
  /// // s.insert(20, 'C');
  /// // s.insert(8, 'D');
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub unsafe fn insert_unchecked(&mut self, index: usize, character: char) {
    let char_length = character.len_utf8();
    shift_right_unchecked(self, index, index + char_length);
    encode_char_utf8_unchecked(self, character, index);
  }

  /// Inserts `character` at `index`, shifting any values that exist in positions greater than
  /// `index` to the right.
  ///
  /// Returns [`StringError::OutOfBounds`] if `character.len_utf8() + self.len()` exceeds the total
  /// capacity of the StaticString and [`StringError::NotCharBoundary`] if `index` does not lie at
  /// a valid UTF-8 character boundary.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// s.try_insert(1, 'E')?;
  /// s.try_insert(2, 'F')?;
  /// assert_eq!(s.as_str(), "AEFBCD🤔");
  /// assert!(s.try_insert(20, 'C').unwrap_err().is_not_char_boundary());
  /// assert!(s.try_insert(8, 'D').unwrap_err().is_not_char_boundary());
  /// let mut s = StaticString::<20>::try_from_str(&"0".repeat(20))?;
  /// assert!(s.try_insert(0, 'C').unwrap_err().is_out_of_bounds());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub fn try_insert(&mut self, index: usize, character: char) -> Result<(), StringError> {
    is_inside_boundary(character.len_utf8() + self.len(), N)?;
    is_char_boundary(self, index)?;
    unsafe { self.insert_unchecked(index, character) };
    Ok(())
  }

  /// Inserts `character` at `index`, shifting any values that exist in positions greater than
  /// `index` to the right.
  ///
  /// Panics if `character.len_utf8() + self.len()` exceeds the total capacity of the StaticString
  /// or if `index` does not lie at a valid UTF-8 character boundary.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString};
  /// let mut s = StaticString::<3>::new();
  /// s.insert(0, 'f');
  /// s.insert(1, 'o');
  /// s.insert(2, 'o');
  /// assert_eq!(s, "foo");
  /// ```
  #[inline(always)]
  pub fn insert(&mut self, index: usize, character: char) {
    self.try_insert(index, character).unwrap();
  }

  /// Inserts `string` at `index`, shifting any values that exist in positions greater than
  /// `index` to the right.
  ///
  /// Does not do any checking to ensure that `self.len() + string.len()` does not exceed
  /// the total capacity of the StaticString or that `index` lies at a valid UTF-8
  /// character boundary.
  ///
  /// # Safety
  ///
  /// `self.len() + string.len()` must not exceed the total capacity of the StaticString instance,
  /// as this would result in writing to an out-of-bounds memory region.
  ///
  /// `Index` must also lie at a valid UTF-8 character boundary, as if it does not, various
  /// assumptions made in the internal implementation of StaticString will be silently
  /// invalidated, almost certainly eventually resulting in undefined behavior.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from_str("ABCD🤔");
  /// unsafe { s.insert_str_unchecked(1, "AB") };
  /// unsafe { s.insert_str_unchecked(1, "BC") };
  /// assert_eq!(s, "ABCABBCD🤔");
  /// // Undefined behavior, don't do it:
  /// // unsafe { s.insert_str_unchecked(20, "C") };
  /// // unsafe { s.insert_str_unchecked(10, "D") };
  /// // unsafe { s.insert_str_unchecked(1, "0".repeat(20)) };
  /// ```
  #[inline]
  pub unsafe fn insert_str_unchecked<S: AsRef<str>>(&mut self, index: usize, string: S) {
    let string_ref = string.as_ref();
    let string_length = string_ref.len();
    debug_assert!(string_length <= self.remaining_capacity());
    let string_ptr = string_ref.as_ptr();
    shift_right_unchecked(self, index, index + string_length);
    string_ptr.copy_to_nonoverlapping(self.vec.mut_ptr_at_unchecked(index), string_length);
    self.vec.set_len(self.len() + string_length);
  }

  /// Inserts `string` at `index`, shifting any values that exist in positions greater than
  /// `index` to the right.
  ///
  /// Panics if `index` is greater than the length of the StaticString or if it does not lie
  /// at a valid UTF-8 character boundary, as well as if `string.len() + self.len()` exceeds
  /// the total capacity of the StaticString.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from("ABCD🤔");
  /// s.insert_str(1, "AB");
  /// s.insert_str(1, "BC");
  /// assert_eq!(s.as_str(), "ABCABBCD🤔");
  /// ```
  #[inline(always)]
  pub fn insert_str<S: AsRef<str>>(&mut self, index: usize, string: S) {
    let string_ref = string.as_ref();
    let string_length = string_ref.len();
    assert!(
      string_length <= self.remaining_capacity() && self.as_str().is_char_boundary(index),
      "Insufficient remaining capacity or invalid character boundary!"
    );
    unsafe { self.insert_str_unchecked(index, string_ref) };
  }

  /// Inserts `string` at `index`, shifting any values that exist in positions greater than
  /// `index` to the right.
  ///
  /// Returns [`StringError::OutOfBounds`] if `self.len() + string.len()` exceeds the total
  /// capacity of the StaticString and [`StringError::NotCharBoundary`] if `index` does not
  /// lie at a valid UTF-8 character boundary.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut string = StaticString::<20>::try_from_str("ABCD🤔")?;
  /// string.try_insert_str(1, "AB")?;
  /// string.try_insert_str(1, "BC")?;
  /// assert!(string.try_insert_str(1, "0".repeat(20)).unwrap_err().is_out_of_bounds());
  /// assert_eq!(string.as_str(), "ABCABBCD🤔");
  /// assert!(string.try_insert_str(20, "C").unwrap_err().is_not_char_boundary());
  /// assert!(string.try_insert_str(10, "D").unwrap_err().is_not_char_boundary());
  /// # Ok(())
  /// # }
  /// ```
  #[inline]
  pub fn try_insert_str<S: AsRef<str>>(
    &mut self,
    index: usize,
    string: S,
  ) -> Result<(), StringError>
  {
    let string_ref = string.as_ref();
    is_inside_boundary(self.len() + string_ref.len(), N)?;
    is_char_boundary(self, index)?;
    unsafe { self.insert_str_unchecked(index, string_ref) };
    Ok(())
  }

  /// Returns the current length of the StaticString.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from("ABCD");
  /// assert_eq!(s.len(), 4);
  /// s.push('🤔');
  /// assert_eq!(s.len(), 8);
  /// ```
  #[inline(always)]
  pub const fn len(&self) -> usize {
    self.vec.len()
  }

  /// Returns true if the StaticString has a current length of 0.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticstring, StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = staticstring!("ABCD");
  /// assert!(!s.is_empty());
  /// s.clear();
  /// assert!(s.is_empty());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub const fn is_empty(&self) -> bool {
    self.vec.is_empty()
  }

  /// Returns true if the StaticString's length is equal to its capacity.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{staticstring, StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = staticstring!("ABCD");
  /// assert!(s.is_full());
  /// s.clear();
  /// assert!(!s.is_full());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub const fn is_full(&self) -> bool {
    self.vec.is_full()
  }

  /// Splits the StaticString in two if `at` is less than the its current length.
  ///
  /// The original StaticString will contain elements `0..at`, and the new one will contain
  /// elements `at..self.len()`.
  ///
  /// Panics if `at` is greater than the length of the StaticString or if it does not
  /// lie at a valid UTF-8 character boundary.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut ab = StaticString::<4>::from("ABCD");
  /// let cd = ab.split_off(2);
  /// assert_eq!(ab, "AB");
  /// assert_eq!(cd, "CD");
  /// ```
  #[inline(always)]
  pub fn split_off(&mut self, at: usize) -> Self {
    assert!(
      at <= self.len() && self.as_str().is_char_boundary(at),
      "Out of bounds or invalid character boundary!"
    );
    unsafe {
      let res = Self::from_utf8_unchecked(self.as_str().get_unchecked(at..));
      self.vec.set_len(at);
      res
    }
  }

  /// Removes all contents from the StaticString and sets its length back to zero.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::{StaticString, StringError};
  /// # fn main() -> Result<(), StringError> {
  /// let mut s = StaticString::<20>::try_from_str("ABCD")?;
  /// assert!(!s.is_empty());
  /// s.clear();
  /// assert!(s.is_empty());
  /// # Ok(())
  /// # }
  /// ```
  #[inline(always)]
  pub fn clear(&mut self) {
    unsafe { self.vec.set_len(0) };
  }

  /// Removes the specified range from the StaticString and replaces it with the provided input
  /// (which does not need to have the same length as the range being removed), panicking if
  /// either the high or low bounds of the range exceed `self.len()` or do not lie at valid UTF-8
  /// character boundaries.
  ///
  /// # Example usage:
  /// ```
  /// # use staticvec::StaticString;
  /// let mut s = StaticString::<20>::from("ABCD🤔");
  /// s.replace_range(2..4, "EFGHI");
  /// assert_eq!(s.as_str(), "ABEFGHI🤔");
  /// ```
  #[inline]
  pub fn replace_range<S: AsRef<str>, R: RangeBounds<usize>>(&mut self, range: R, with: S) {
    let replace_with = with.as_ref();
    let old_length = self.len();
    let start = match range.start_bound() {
      Bound::Included(t) => *t,
      Bound::Excluded(t) => t + 1,
      Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
      Bound::Included(t) => t + 1,
      Bound::Excluded(t) => *t,
      Bound::Unbounded => old_length,
    };
    let replace_length = replace_with.len();
    assert!(
      start <= end && end <= old_length,
      "Invalid range or out of bounds!"
    );
    let replaced = end.saturating_sub(start);
    assert!(
      replaced + replace_length <= N
        && self.as_str().is_char_boundary(start)
        && self.as_str().is_char_boundary(end),
      "Out of bounds or invalid character boundary!"
    );
    if replace_length == 0 {
      unsafe {
        self
          .as_ptr()
          .add(end)
          .copy_to(self.as_mut_ptr().add(start), old_length.saturating_sub(end));
        self.vec.set_len(old_length.saturating_sub(replaced));
      }
    } else {
      if start + replace_length > end {
        unsafe { shift_right_unchecked(self, end, start + replace_length) };
      } else {
        unsafe { shift_left_unchecked(self, end, start + replace_length) };
      }
      let ptr = replace_with.as_ptr();
      let dest = unsafe { self.vec.as_mut_ptr().add(start) };
      unsafe { ptr.copy_to_nonoverlapping(dest, replace_length) };
      let grow: isize = replace_length as isize - replaced as isize;
      unsafe { self.vec.set_len((old_length as isize + grow) as usize) };
    }
  }
}
