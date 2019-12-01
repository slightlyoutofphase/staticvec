/// Creates a new StaticVec from a [`vec!`](https://doc.rust-lang.org/nightly/alloc/macro.vec.html)-style pseudo-slice.
/// The newly created StaticVec will have a capacity and length exactly equal to the
/// number of elements in the slice. The "array-like" `[value; N]` syntax is also supported for
/// types that implement [`Copy`](core::marker::Copy).
///
/// Example usage:
///
/// ```
/// // The type of the StaticVec on the next line is `StaticVec<Vec<StaticVec<i32, 4>>, 1>`.
/// let v = staticvec![vec![staticvec![1, 2, 3, 4]]];
/// // The type of the StaticVec on the next line is `StaticVec<f32, 64>`.
/// let v2 = staticvec![12.0; 64];
/// ```
#[macro_export]
macro_rules! staticvec {
  (@put_one $val:expr) => {
    1
  };
  ($val:expr; $n:expr) => {
    $crate::utils::new_from_value::<_, $n>($val)
  };
  ($($val:expr),* $(,)*) => {
    staticvec::StaticVec::<_, {0$(+staticvec::staticvec!(@put_one $val))*}>::new_from_array([$($val),*])
  };
}

macro_rules! impl_extend {
  ($var_a:tt, $var_b:tt, $type:ty) => {
    /// Appends all elements, if any, from `iter` to the StaticVec. If `iter` has a size greater than
    /// the StaticVec's capacity, any items after that point are ignored.
    #[inline]
    fn extend<I: IntoIterator<Item = $type>>(&mut self, iter: I) {
      let mut it = iter.into_iter();
      let mut i = self.length;
      while i < N {
        if let Some($var_a) = it.next() {
          unsafe {
            self.data.get_unchecked_mut(i).write($var_b);
          }
        } else {
          break;
        }
        i += 1;
      }
      self.length = i;
    }
  };
}

macro_rules! impl_from_iterator {
  ($var_a:tt, $var_b:tt, $type:ty) => {
    /// Creates a new StaticVec instance from the elements, if any, of `iter`.
    /// If `iter` has a size greater than the StaticVec's capacity, any items after
    /// that point are ignored.
    #[inline]
    fn from_iter<I: IntoIterator<Item = $type>>(iter: I) -> Self {
      let mut i = 0;
      Self {
        data: {
          let mut res = Self::new_data_uninit();
          let mut it = iter.into_iter();
          while i < N {
            if let Some($var_a) = it.next() {
              unsafe {
                (res.as_mut_ptr() as *mut T).add(i).write($var_b);
              }
            } else {
              break;
            }
            i += 1;
          }
          unsafe { res.assume_init() }
        },
        length: i,
      }
    }
  };
}

macro_rules! impl_partial_eq_with_as_slice {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N1: usize, const N2: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        self.as_slice() == other.as_slice()
      }
      #[allow(clippy::partialeq_ne_impl)]
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        self.as_slice() != other.as_slice()
      }
    }
  };
}

macro_rules! impl_partial_eq_with_get_unchecked {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N1: usize, const N2: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        unsafe { self.as_slice() == other.get_unchecked(..) }
      }
      #[allow(clippy::partialeq_ne_impl)]
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        unsafe { self.as_slice() != other.get_unchecked(..) }
      }
    }
  };
}

macro_rules! impl_partial_eq_with_equals_no_deref {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        self.as_slice() == other
      }
      #[allow(clippy::partialeq_ne_impl)]
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        self.as_slice() != other
      }
    }
  };
}

macro_rules! impl_partial_eq_with_equals_deref {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialEq<T1>, const N: usize> PartialEq<$left> for $right {
      #[inline(always)]
      fn eq(&self, other: &$left) -> bool {
        self.as_slice() == *other
      }
      #[allow(clippy::partialeq_ne_impl)]
      #[inline(always)]
      fn ne(&self, other: &$left) -> bool {
        self.as_slice() != *other
      }
    }
  };
}

macro_rules! impl_partial_ord_with_as_slice {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialOrd<T1>, const N1: usize, const N2: usize> PartialOrd<$left> for $right {
      #[inline(always)]
      fn partial_cmp(&self, other: &$left) -> Option<Ordering> {
        partial_compare(self.as_slice(), other.as_slice())
      }
    }
  };
}

macro_rules! impl_partial_ord_with_get_unchecked {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialOrd<T1>, const N1: usize, const N2: usize> PartialOrd<$left> for $right {
      #[inline(always)]
      fn partial_cmp(&self, other: &$left) -> Option<Ordering> {
        unsafe { partial_compare(self.as_slice(), other.get_unchecked(..)) }
      }
    }
  };
}

macro_rules! impl_partial_ord_with_as_slice_against_slice {
  ($left:ty, $right:ty) => {
    impl<T1, T2: PartialOrd<T1>, const N: usize> PartialOrd<$left> for $right {
      #[inline(always)]
      fn partial_cmp(&self, other: &$left) -> Option<Ordering> {
        partial_compare(self.as_slice(), other)
      }
    }
  };
}
