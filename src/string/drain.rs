//! Draining iterator for [`StaticString`]
//!
//! [`StaticString`]: ../struct.StaticString.html

use super::StaticString;
use core::fmt::{self, Debug, Formatter};
use core::{cmp::Ordering, hash::Hash, hash::Hasher, iter::FusedIterator};

/// A draining iterator for [`StaticString`].
///
/// Created through [`drain`]
///
/// [`StaticString`]: ../struct.StaticString.html
/// [`drain`]: ../struct.StaticString.html#method.drain
#[derive(Clone, Default)]
pub struct Drain<const N: usize>(pub(crate) StaticString<N>, pub(crate) usize);

impl<const N: usize> Debug for Drain<N> {
  #[inline]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("Drain")
      .field(&self.0)
      .field(&self.1)
      .finish()
  }
}

impl<const N: usize> PartialEq for Drain<N> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.as_str().eq(other.as_str())
  }
}
impl<const N: usize> Eq for Drain<N> {}

impl<const N: usize> Ord for Drain<N> {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl<const N: usize> PartialOrd for Drain<N> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<const N: usize> Hash for Drain<N> {
  #[inline]
  fn hash<H: Hasher>(&self, hasher: &mut H) {
    self.as_str().hash(hasher)
  }
}

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
