use core::fmt;

#[cfg(feature = "std")]
use std::error;

/// This error indicates that an operation was attempted that would have increased the
/// `length` value of a [`StaticVec`](crate::StaticVec), but the [`StaticVec`](crate::StaticVec) was
/// already at its maximum capacity of `N`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CapacityError<const N: usize>;

impl<const N: usize> fmt::Display for CapacityError<N> {
  #[inline(always)]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Insufficient remaining capacity (limit is {})!", N)
  }
}

#[cfg(feature = "std")]
impl<const N: usize> error::Error for CapacityError<N> {}

/// This error indicates that a push was attempted into a
/// [`StaticVec`](crate::StaticVec) which failed because the
/// [`StaticVec`](crate::StaticVec) was already at maximum capacity. It contains the value
/// that failed to be pushed so that it can be reused if needed.
#[derive(Clone, Eq, PartialEq)]
pub struct PushCapacityError<T, const N: usize>(T);

impl<T, const N: usize> PushCapacityError<T, N> {
  #[inline(always)]
  pub(crate) fn new(value: T) -> Self {
    PushCapacityError(value)
  }

  /// Extracts the failed value from the error.
  #[inline(always)]
  pub fn into_value(self) -> T {
    self.0
  }
}

impl<T, const N: usize> AsRef<T> for PushCapacityError<T, N> {
  #[inline(always)]
  fn as_ref(&self) -> &T {
    &self.0
  }
}

impl<T, const N: usize> AsMut<T> for PushCapacityError<T, N> {
  #[inline(always)]
  fn as_mut(&mut self) -> &mut T {
    &mut self.0
  }
}

impl<T, const N: usize> fmt::Display for PushCapacityError<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // The unpushed value isn't really relevant to the error, so we don't
    // print it.
    write!(
      f,
      "Insufficient remaining capacity for push (limit is {})",
      N
    )
  }
}

impl<T, const N: usize> fmt::Debug for PushCapacityError<T, N> {
  #[inline(always)]
  default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("PushCapacityError")
      .field("N", &N)
      .field("value", &"...")
      .finish()
  }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for PushCapacityError<T, N> {
  #[inline(always)]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("PushCapacityError")
      .field("N", &N)
      .field("value", &self.0)
      .finish()
  }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug, const N: usize> error::Error for PushCapacityError<T, N> {
  #[inline(always)]
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    Some(&CapacityError::<N>)
  }
}
