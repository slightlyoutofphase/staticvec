///Creates a new StaticVec from a `vec!`-style macro slice, using [`new_from_slice`](crate::StaticVec::new_from_slice)
///internally. The newly created StaticVec will have a `capacity` and `length` exactly equal
///to the number of elements in the slice.
#[macro_export]
macro_rules! staticvec {
  (@add_one $x:expr) => (1);
  ($($x:expr),*$(,)*) => ({
    const CAP: usize = 0 $(+ staticvec!(@add_one $x))*;
    fn new_from_temp_slice<T>(values: &[T]) -> StaticVec<T, { CAP }> {
      unsafe {
        let mut _data: [MaybeUninit<T>; CAP] = MaybeUninit::uninit().assume_init();
        values
          .as_ptr()
          .copy_to_nonoverlapping(_data.as_mut_ptr() as *mut T, CAP);
        StaticVec::<T, { CAP }> {
          data: _data,
          length: CAP,
        }
      }
    }
    new_from_temp_slice::<_>(&[$($x,)*])
  });
}