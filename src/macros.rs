/// Creates a new [`StaticVec`](crate::StaticVec) from a [`vec!`](alloc::vec::Vec)-style
/// pseudo-slice. The newly created [`StaticVec`](crate::StaticVec) will have a capacity and length
/// exactly equal to the number of elements in the slice. The "array-like" `[value; N]` syntax is
/// also supported, and both forms can be used in const contexts.
///
/// Example usage:
///
/// ```
/// // The type of the StaticVec on the next line is `StaticVec<Vec<StaticVec<i32, 4>>, 1>`.
/// let v = staticvec![vec![staticvec![1, 2, 3, 4]]];
/// // The type of the StaticVec on the next line is `StaticVec<f64, 64>`.
/// let v2 = staticvec![12.0; 64];
/// const V3: StaticVec<i32, 4> = staticvec![1, 2, 3, 4];
/// assert_eq!(V3, [1, 2, 3, 4]);
/// const V4: StaticVec<i32, 128> = staticvec![27; 128];
/// assert!(V4 == [27; 128]);
/// ```
#[macro_export]
macro_rules! staticvec {
  ($val:expr; $n:expr) => {
    $crate::StaticVec::new_from_const_array([$val; $n])
  };
  ($($val:expr),* $(,)*) => {
    $crate::StaticVec::new_from_const_array([$($val),*])
  };
}

/// Accepts an array of any [`Copy`](core::marker::Copy) type that derives or implements
/// [`PartialOrd`](core::cmp::PartialOrd), sorts it, and creates a new
/// [`StaticVec`](crate::StaticVec) instance from the result in a fully const context compatible
/// manner.
///
/// Example usage:
///
/// ```
/// #![feature(const_fn, const_if_match, const_loop)]
/// // Currently, it's necessary to have the type specified in the macro itself.
/// static V: StaticVec<f64, 3> = sortedstaticvec!(f64, [16.0, 15.0, 14.0]);
/// assert_eq!(V, [14.0, 15.0, 16.0]);
/// assert_eq!(V.reversed().drain(0..1), [16.0]);
/// static VV: StaticVec<f64, 0> = sortedstaticvec!(f64, []);
/// assert_eq!(VV, []);
/// ```
#[macro_export]
macro_rules! sortedstaticvec {
  (@put_one $val:expr) => (1);
  ($type: ty, [$($val:expr),* $(,)*]) => {{
    #[doc(hidden)]
    use staticsort::staticsort;
    match 0$(+sortedstaticvec!(@put_one $val))* {
      0 => $crate::StaticVec::new(),
      _ => $crate::StaticVec::new_from_const_array(
             staticsort!(
               $type,
               0,
               0$(+sortedstaticvec!(@put_one $val))* - 1,
               0$(+sortedstaticvec!(@put_one $val))*,
               [$($val),*]
             )
           ),
    }

  };};
}

macro_rules! impl_extend_ex {
  ($var_a:tt, $var_b:tt) => {
    /// Appends all elements, if any, from `iter` to the StaticVec. If `iter` has a size greater than
    /// the StaticVec's capacity, any items after that point are ignored.
    #[allow(unused_parens)]
    #[inline]
    default fn extend_ex(&mut self, iter: I) {
      let mut it = iter.into_iter();
      let mut i = self.length;
      while i < N {
        if let Some($var_a) = it.next() {
          unsafe {
            self.mut_ptr_at_unchecked(i).write($var_b);
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

macro_rules! impl_from_iter_ex {
  ($var_a:tt, $var_b:tt) => {
    /// Creates a new StaticVec instance from the elements, if any, of `iter`.
    /// If `iter` has a size greater than the StaticVec's capacity, any items after
    /// that point are ignored.
    #[allow(unused_parens)]
    #[inline]
    default fn from_iter_ex(iter: I) -> Self {
      let mut res = Self::new_data_uninit();
      let mut it = iter.into_iter();
      let mut i = 0;
      while i < N {
        if let Some($var_a) = it.next() {
          unsafe {
            Self::first_ptr_mut(&mut res).add(i).write($var_b);
          }
        } else {
          break;
        }
        i += 1;
      }
      Self {
        data: res,
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
