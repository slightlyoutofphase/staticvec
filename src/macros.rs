///Creates a new StaticVec from a `vec!`-style macro-slice.
///The newly created StaticVec will have a `capacity` and `length` exactly equal to the
///number of elements, if any, in the slice.
#[macro_export]
macro_rules! staticvec {
  (@add_one $x:expr) => (1);
  ($($x:expr),*$(,)*) => ({
    use staticvec::macro_constructor::__new_from_temp_slice;
    const CAP: usize = 0$(+staticvec!(@add_one $x))*;
    unsafe { __new_from_temp_slice::<_,{CAP}>(&[$($x,)*]) }
  });
}
