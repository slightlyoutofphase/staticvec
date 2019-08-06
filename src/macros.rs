///Creates a new StaticVec from a `vec!`-style pseudo-slice.
///The newly created StaticVec will have a `capacity` and `length` exactly equal to the
///number of elements, if any, in the slice.
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
