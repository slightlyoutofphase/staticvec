/// Creates a new [`StaticVec`](crate::StaticVec) from a [`vec!`](alloc::vec::Vec)-style
/// pseudo-slice. The newly created [`StaticVec`](crate::StaticVec) will have a capacity and length
/// exactly equal to the number of elements in the so-called slice. The "array-like" `[value; N]`
/// syntax is also supported, and both forms can be used in const contexts.
///
/// Example usage:
/// ```
/// use staticvec::{staticvec, StaticVec};
///
/// // The type of the StaticVec on the next line is `StaticVec<Vec<StaticVec<i32, 4>>, 1>`.
/// let v = staticvec![vec![staticvec![1, 2, 3, 4]]];
///
/// // The type of the StaticVec on the next line is `StaticVec<f64, 64>`.
/// let v2 = staticvec![12.0; 64];
///
/// const V3: StaticVec<i32, 4> = staticvec![1, 2, 3, 4];
/// assert_eq!(V3, [1, 2, 3, 4]);
///
/// static V4: StaticVec<i32, 128> = staticvec![27; 128];
/// assert!(V4 == [27; 128]);
/// ```
#[macro_export]
macro_rules! staticvec {
  ($($val:expr),* $(,)*) => {
    $crate::StaticVec::new_from_const_array([$($val),*])
  };
  ($val:expr; $n:expr) => {
    $crate::StaticVec::new_from_const_array([$val; $n])
  };
}

/// Creates a new [`StaticString`] from an `&str` literal. This macro can be used in const
/// contexts, in keeping with the other ones in this crate.
///
/// The `staticstring!` macro comes in two forms: and comes in two forms: one that solely
/// takes an `&str` literal, where the resulting [`StaticString`] will have a total
/// capacity exactly equal to the number of bytes in the literal, and one that takes an
/// additional integral constant which is then used to specify the capacity independently
/// from the length of the input string.
///
/// Implemententing it as such allows the macro be more flexible than would otherwise
/// be possible due to the required level of type inference being beyond what the compiler
/// is (currently at least) capable of.
///
/// Example usage:
/// ```
/// # use staticvec::*;
/// // Usage at runtime, creating a `StaticString` with both a length and capacity of 10:
/// let s1 = staticstring!("ABCDEFGHIJ");
///
/// // Usage at runtime, creating a `StaticString` with a length of 10 but a capacity of 20:
/// let s2 = staticstring!("ABCDEFGHIJ", 20);
///
/// // Usage at compile time, creating a `StaticString` with both a length and capacity of 10:
/// const S3: StaticString<10> = staticstring!("ABCDEFGHIJ");
///
/// // Usage at compile time, creating a `StaticString` with a length of 18 but a capacity of 36,
/// // keeping in mind that length is measured in bytes and not characters of course:
/// const S4: StaticString<18> = staticstring!("BC🤔BC🤔BC🤔", 36);
/// ```
///
/// Note that attempting to explicitly provide a capacity that is less than the number of bytes
/// in the input string will give a *compile-time* index error in const contexts, and a regular
/// index error panic in the context of runtime usage.
///
/// For example, this would give a compile-time error:
/// ```
/// const S5: StaticString<1> = staticstring!("ABCDEFG");
/// ```
/// And this would panic with an index error at runtime:
/// ```
/// let s6: StaticString<0> = staticstring!("🤔🤔🤔🤔🤔🤔");
/// ```
///
/// In the future, it may be possible to somehow catch everything at compile time directly within
/// the macro invocation, but for the time being allowing the index error to occur is the safest
/// route to take with the implementation as it does properly prevent `StaticString` instances
/// containing invalid UTF-8 from ever being constructed via the macro.
#[macro_export]
#[rustfmt::skip]
macro_rules! staticstring {
  ($val:expr) => {{
    const CAP: usize = $val.len();
    $crate::StaticString::<CAP>::__new_from_staticvec(
      $crate::StaticVec::<u8, CAP>::from_const_str($val)
    )
  };};
  ($val:expr, $n:expr) => {{
    const CAP: usize = $n;
    $crate::StaticString::<CAP>::__new_from_staticvec(
      $crate::StaticVec::<u8, CAP>::from_const_str($val)
    )
  };};
}

/// This is the same macro available in my actual `staticsort` crate, which I previously had as
/// a dependency for this crate but decided to "inline" here as considering I wrote it myself it
/// seems silly to have a mandatory dependency for no real reason.
#[doc(hidden)]
#[macro_export]
macro_rules! __staticsort {
  ($type:ty, $low:expr, $high:expr, $len:expr, $values:expr) => {{
    match $len {
      0 => $values,
      _ => {
        #[inline]
        const fn static_sort(
          mut values: [$type; $len],
          mut low: isize,
          mut high: isize,
        ) -> [$type; $len]
        {
          if high - low <= 0 {
            return values;
          }
          loop {
            let mut i = low;
            let mut j = high;
            let p = values[(low + ((high - low) >> 1)) as usize];
            loop {
              while values[i as usize] < p {
                i += 1;
              }
              while values[j as usize] > p {
                j -= 1;
              }
              if i <= j {
                if i != j {
                  let q = values[i as usize];
                  values[i as usize] = values[j as usize];
                  values[j as usize] = q;
                }
                i += 1;
                j -= 1;
              }
              if i > j {
                break;
              }
            }
            if j - low < high - i {
              if low < j {
                values = static_sort(values, low, j);
              }
              low = i;
            } else {
              if i < high {
                values = static_sort(values, i, high)
              }
              high = j;
            }
            if low >= high {
              break;
            }
          }
          values
        }
        static_sort($values, $low, $high)
      }
    }
  };};
}

/// Accepts an array of any primitive [`Copy`](core::marker::Copy) type that has a
/// [`PartialOrd`](core::cmp::PartialOrd) implementation, sorts it, and creates a new
/// [`StaticVec`](crate::StaticVec) instance from the result in a fully const context compatible
/// manner.
///
/// Example usage:
/// ```
/// #![feature(const_fn, const_if_match, const_loop)]
///
/// use staticvec::{sortedstaticvec, StaticVec};
///
/// // Currently, it's necessary to have the type specified in the macro itself.
/// static V: StaticVec<f64, 3> = sortedstaticvec!(f64, [16.0, 15.0, 14.0]);
/// assert_eq!(V, [14.0, 15.0, 16.0]);
///
/// const V2: StaticVec<usize, 4> = sortedstaticvec!(usize, [16, 15, 14, 13]);
/// assert_eq!(V2, [13, 14, 15, 16]);
/// ```
#[macro_export]
macro_rules! sortedstaticvec {
  (@put_one $val:expr) => (1);
  ($type: ty, [$($val:expr),* $(,)*]) => {{
    const LEN: usize = 0$(+sortedstaticvec!(@put_one $val))*;
    match LEN {
      0 => $crate::StaticVec::new(),
      _ => $crate::StaticVec::new_from_const_array(
             $crate::__staticsort!(
               $type,
               0,
               (LEN as isize) - 1,
               LEN,
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
