//! Draining iterator for [`StaticString`]
//!
//! [`StaticString`]: ../struct.StaticString.html

use super::StaticString;
use core::iter::FusedIterator;

/// A draining iterator for [`StaticString`].
///
/// Created through [`drain`]
///
/// [`StaticString`]: ../struct.StaticString.html
/// [`drain`]: ../struct.StaticString.html#method.drain
#[derive(Debug)]
pub struct Drain<const N: usize>(pub(crate) StaticString<N>, pub(crate) usize);

impl<const N: usize> Drain<N> {
  /// Extracts string slice containing the remaining characters of `Drain`.
  #[inline]
  pub fn as_str(&self) -> &str {
    unsafe { self.0.as_str().get_unchecked(self.1.into()..) }
  }
}

impl<const N: usize> Iterator for Drain<N> {
  type Item = char;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self
      .0
      .as_str()
      .get(self.1.into()..)
      .and_then(|s| s.chars().next())
      .map(|c| {
        self.1 = self.1.saturating_add(c.len_utf8());
        c
      })
  }
}

impl<const N: usize> DoubleEndedIterator for Drain<N> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    self.0.pop()
  }
}

impl<const N: usize> FusedIterator for Drain<N> {}
