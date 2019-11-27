use crate::utils::distance_between;

#[cfg(feature = "std")]
use core::fmt::Debug;

#[cfg(feature = "std")]
use alloc::string::String;

#[cfg(feature = "std")]
use alloc::format;

use core::intrinsics;
use core::iter::{FusedIterator, TrustedLen};
use core::marker::{PhantomData, Send, Sync};

/// Similar to [Iter](core::slice::IterMut), but specifically implemented with StaticVecs in mind.
pub struct StaticVecIterConst<'a, T: 'a> {
  pub(crate) start: *const T,
  pub(crate) end: *const T,
  pub(crate) marker: PhantomData<&'a T>,
}

/// Similar to [IterMut](core::slice::IterMut), but specifically implemented with StaticVecs in
/// mind.
pub struct StaticVecIterMut<'a, T: 'a> {
  pub(crate) start: *mut T,
  pub(crate) end: *mut T,
  pub(crate) marker: PhantomData<&'a mut T>,
}

impl<'a, T: 'a> StaticVecIterConst<'a, T> {
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    // Safety: `start` and `end` are never null.
    unsafe {
      format!(
        "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
        *self.start, *self.end
      )
    }
  }
}

impl<'a, T: 'a> Iterator for StaticVecIterConst<'a, T> {
  type Item = &'a T;
  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    if (self.start as usize) < (self.end as usize) {
      unsafe {
        let res = Some(&*self.start);
        self.start = if intrinsics::size_of::<T>() == 0 {
          (self.start as usize + 1) as *const _
        } else {
          self.start.offset(1)
        };
        res
      }
    } else {
      None
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = distance_between(self.end, self.start);
    (len, Some(len))
  }
}

impl<'a, T: 'a> DoubleEndedIterator for StaticVecIterConst<'a, T> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<Self::Item> {
    if (self.end as usize) > (self.start as usize) {
      unsafe {
        self.end = if intrinsics::size_of::<T>() == 0 {
          (self.end as usize - 1) as *const _
        } else {
          self.end.offset(-1)
        };
        Some(&*self.end)
      }
    } else {
      None
    }
  }
}

impl<'a, T: 'a> ExactSizeIterator for StaticVecIterConst<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a> FusedIterator for StaticVecIterConst<'a, T> {}
unsafe impl<'a, T: 'a> TrustedLen for StaticVecIterConst<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Sync for StaticVecIterConst<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Send for StaticVecIterConst<'a, T> {}

impl<'a, T: 'a + Debug> StaticVecIterMut<'a, T> {
  #[cfg(feature = "std")]
  #[doc(cfg(feature = "std"))]
  #[inline(always)]
  /// Returns a string displaying the current values of the
  /// iterator's `start` and `end` elements on two separate lines.
  /// Locally requires that `T` implements [Debug](core::fmt::Debug)
  /// to make it possible to pretty-print the elements.
  pub fn bounds_to_string(&self) -> String
  where T: Debug {
    // Safety: `start` and `end` are never null.
    unsafe {
      format!(
        "Current value of element at `start`: {:?}\nCurrent value of element at `end`: {:?}",
        *self.start, *self.end
      )
    }
  }
}

impl<'a, T: 'a> Iterator for StaticVecIterMut<'a, T> {
  type Item = &'a mut T;
  #[inline(always)]
  fn next(&mut self) -> Option<Self::Item> {
    if (self.start as usize) < (self.end as usize) {
      unsafe {
        let res = Some(&mut *self.start);
        self.start = if intrinsics::size_of::<T>() == 0 {
          (self.start as usize + 1) as *mut _
        } else {
          self.start.offset(1)
        };
        res
      }
    } else {
      None
    }
  }

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = distance_between(self.end, self.start);
    (len, Some(len))
  }
}

impl<'a, T: 'a> DoubleEndedIterator for StaticVecIterMut<'a, T> {
  #[inline(always)]
  fn next_back(&mut self) -> Option<Self::Item> {
    if (self.end as usize) > (self.start as usize) {
      unsafe {
        self.end = if intrinsics::size_of::<T>() == 0 {
          (self.end as usize - 1) as *mut _
        } else {
          self.end.offset(-1)
        };
        Some(&mut *self.end)
      }
    } else {
      None
    }
  }
}

impl<'a, T: 'a> ExactSizeIterator for StaticVecIterMut<'a, T> {
  #[inline(always)]
  fn len(&self) -> usize {
    distance_between(self.end, self.start)
  }

  #[inline(always)]
  fn is_empty(&self) -> bool {
    distance_between(self.end, self.start) == 0
  }
}

impl<'a, T: 'a> FusedIterator for StaticVecIterMut<'a, T> {}
unsafe impl<'a, T: 'a> TrustedLen for StaticVecIterMut<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Sync for StaticVecIterMut<'a, T> {}
unsafe impl<'a, T: 'a + Sync> Send for StaticVecIterMut<'a, T> {}
