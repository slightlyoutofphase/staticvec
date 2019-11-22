///Creates a new StaticVec from a `vec!`-style pseudo-slice.
///The newly created StaticVec will have a `capacity` and `length` exactly equal to the
///number of elements in the slice. The "array-like" `[value; N]` syntax is also supported
///for types that implement `Copy`.
#[macro_export]
macro_rules! staticvec {
  (@put_one $val:expr) => (1);
  ($val:expr; $n:expr) => (
    $crate::utils::new_from_value::<_, $n>($val)
  );
  ($($val:expr),*$(,)*) => ({
    let mut res = StaticVec::<_, {0$(+staticvec!(@put_one $val))*}>::new();
    {
      unsafe {
        ($({
          res.push_unchecked($val);
        }),*)
      }
    };
    res
  });
}

macro_rules! impl_extend {
  ($var_a:tt, $var_b:tt, $type:ty) => {
    ///Appends all elements, if any, from `iter` to the StaticVec. If `iter` has a size greater than
    ///the StaticVec's capacity, any items after that point are ignored.
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
    ///Creates a new StaticVec instance from the elements, if any, of `iter`.
    ///If `iter` has a size greater than the StaticVec's capacity, any items after
    ///that point are ignored.
    #[inline]
    fn from_iter<I: IntoIterator<Item = $type>>(iter: I) -> Self {
      let mut res = Self::new();
      let mut it = iter.into_iter();
      let mut i = 0;
      while i < N {
        if let Some($var_a) = it.next() {
          unsafe {
            res.data.get_unchecked_mut(i).write($var_b);
          }
        } else {
          break;
        }
        i += 1;
      }
      res.length = i;
      res
    }
  };
}
