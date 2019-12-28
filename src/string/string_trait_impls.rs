use super::StaticString;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt::{self, Debug, Display, Formatter, Write};
use core::hash::{Hash, Hasher};
use core::iter::FromIterator;
use core::ops::{
  Add, Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
  RangeToInclusive,
};
use core::str::{self, FromStr};

impl<'a, const N: usize> Add<&'a str> for StaticString<N> {
  type Output = Self;

  #[inline(always)]
  fn add(mut self, other: &str) -> Self::Output {
    self.push_str(other);
    self
  }
}

impl<const N: usize> AsMut<str> for StaticString<N> {
  #[inline(always)]
  fn as_mut(&mut self) -> &mut str {
    self.as_mut_str()
  }
}

impl<const N: usize> AsRef<str> for StaticString<N> {
  #[inline(always)]
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl<const N: usize> AsRef<[u8]> for StaticString<N> {
  #[inline(always)]
  fn as_ref(&self) -> &[u8] {
    self.as_bytes()
  }
}

impl<const N: usize> Borrow<str> for StaticString<N> {
  #[inline(always)]
  fn borrow(&self) -> &str {
    self.as_str()
  }
}

impl<const N: usize> BorrowMut<str> for StaticString<N> {
  #[inline(always)]
  fn borrow_mut(&mut self) -> &mut str {
    self.as_mut_str()
  }
}

impl<const N: usize> Debug for StaticString<N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_struct("StaticString")
      .field("array", &self.as_str())
      .field("size", &self.len())
      .finish()
  }
}

impl<const N: usize> Default for StaticString<N> {
  #[inline(always)]
  fn default() -> Self {
    Self::new()
  }
}

impl<const N: usize> Deref for StaticString<N> {
  type Target = str;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    self.as_ref()
  }
}

impl<const N: usize> DerefMut for StaticString<N> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut()
  }
}

impl<const N: usize> Display for StaticString<N> {
  #[inline(always)]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl<const N: usize> Eq for StaticString<N> {}

impl<const N: usize> Extend<char> for StaticString<N> {
  #[inline(always)]
  fn extend<I: IntoIterator<Item = char>>(&mut self, iterable: I) {
    self.push_str(Self::from_chars(iterable))
  }
}

impl<'a, const N: usize> Extend<&'a char> for StaticString<N> {
  #[inline(always)]
  fn extend<I: IntoIterator<Item = &'a char>>(&mut self, iter: I) {
    self.extend(iter.into_iter().copied());
  }
}

impl<'a, const N: usize> Extend<&'a str> for StaticString<N> {
  #[inline(always)]
  fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iterable: I) {
    self.push_str(Self::from_iterator(iterable))
  }
}

impl<'a, const N: usize> From<&'a str> for StaticString<N> {
  #[inline(always)]
  fn from(s: &str) -> Self {
    Self::from_str(s)
  }
}

impl<const N: usize> FromIterator<char> for StaticString<N> {
  #[inline(always)]
  fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
    Self::from_chars(iter)
  }
}

impl<'a, const N: usize> FromIterator<&'a str> for StaticString<N> {
  #[inline(always)]
  fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
    Self::from_iterator(iter)
  }
}

impl<'a, const N: usize> FromStr for StaticString<N> {
  type Err = ();

  #[inline(always)]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from_str(s))
  }
}

impl<const N: usize> Hash for StaticString<N> {
  #[inline(always)]
  fn hash<H: Hasher>(&self, hasher: &mut H) {
    self.as_str().hash(hasher);
  }
}

impl<const N: usize> Index<Range<usize>> for StaticString<N> {
  type Output = str;

  #[inline(always)]
  fn index(&self, index: Range<usize>) -> &Self::Output {
    let (start, end) = (index.start as usize, index.end as usize);
    self.as_str().index(Range { start, end })
  }
}

impl<const N: usize> IndexMut<Range<usize>> for StaticString<N> {
  #[inline(always)]
  fn index_mut(&mut self, index: Range<usize>) -> &mut str {
    let (start, end) = (index.start as usize, index.end as usize);
    let range = Range { start, end };
    self.as_mut_str().index_mut(range)
  }
}

impl<const N: usize> Index<RangeFrom<usize>> for StaticString<N> {
  type Output = str;

  #[inline(always)]
  fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
    let start = index.start as usize;
    self.as_str().index(RangeFrom { start })
  }
}

impl<const N: usize> IndexMut<RangeFrom<usize>> for StaticString<N> {
  #[inline(always)]
  fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut str {
    let start = index.start as usize;
    self.as_mut_str().index_mut(RangeFrom { start })
  }
}

impl<const N: usize> Index<RangeFull> for StaticString<N> {
  type Output = str;

  #[inline(always)]
  fn index(&self, index: RangeFull) -> &Self::Output {
    self.as_str().index(index)
  }
}

impl<const N: usize> IndexMut<RangeFull> for StaticString<N> {
  #[inline(always)]
  fn index_mut(&mut self, index: RangeFull) -> &mut str {
    self.as_mut_str().index_mut(index)
  }
}

impl<const N: usize> Index<RangeInclusive<usize>> for StaticString<N> {
  type Output = str;

  #[inline(always)]
  fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
    let (start, end) = (*index.start(), *index.end());
    let range = RangeInclusive::new(start, end);
    self.as_str().index(range)
  }
}

impl<const N: usize> IndexMut<RangeInclusive<usize>> for StaticString<N> {
  #[inline(always)]
  fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut str {
    let (start, end) = (*index.start() as usize, *index.end() as usize);
    let range = RangeInclusive::new(start, end);
    self.as_mut_str().index_mut(range)
  }
}

impl<const N: usize> Index<RangeTo<usize>> for StaticString<N> {
  type Output = str;

  #[inline(always)]
  fn index(&self, index: RangeTo<usize>) -> &Self::Output {
    let end = index.end as usize;
    self.as_str().index(RangeTo { end })
  }
}

impl<const N: usize> IndexMut<RangeTo<usize>> for StaticString<N> {
  #[inline(always)]
  fn index_mut(&mut self, index: RangeTo<usize>) -> &mut str {
    let end = index.end as usize;
    self.as_mut_str().index_mut(RangeTo { end })
  }
}

impl<const N: usize> Index<RangeToInclusive<usize>> for StaticString<N> {
  type Output = str;

  #[inline(always)]
  fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
    let end = index.end as usize;
    self.as_str().index(RangeToInclusive { end })
  }
}

impl<const N: usize> IndexMut<RangeToInclusive<usize>> for StaticString<N> {
  #[inline(always)]
  fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut str {
    let end = index.end as usize;
    let range = RangeToInclusive { end };
    self.as_mut_str().index_mut(range)
  }
}

impl<const N: usize> Ord for StaticString<N> {
  #[inline(always)]
  fn cmp(&self, other: &Self) -> Ordering {
    self.as_str().cmp(other.as_str())
  }
}

impl<const N: usize> PartialEq for StaticString<N> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.as_str().eq(other.as_str())
  }
}

impl<'a, 'b, const N: usize> PartialEq<str> for StaticString<N> {
  #[inline(always)]
  fn eq(&self, other: &str) -> bool {
    self.as_str().eq(other)
  }
}

impl<const N: usize> PartialOrd for StaticString<N> {
  #[inline(always)]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<const N: usize> Write for StaticString<N> {
  #[inline(always)]
  fn write_str(&mut self, slice: &str) -> fmt::Result {
    self.try_push_str(slice).map_err(|_| fmt::Error)
  }
}
