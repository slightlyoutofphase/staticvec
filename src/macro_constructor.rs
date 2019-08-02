use crate::StaticVec;
use std::mem::MaybeUninit;

#[doc(hidden)]
#[inline(always)]
pub fn __new_from_temp_slice<T, const N: usize>(values: &[T]) -> StaticVec<T, {N}> {
  unsafe {
    let mut _data: [MaybeUninit<T>; N] = MaybeUninit::uninit().assume_init();
    values
      .as_ptr()
      .copy_to_nonoverlapping(_data.as_mut_ptr() as *mut T, N);
    StaticVec::<T, {N}> {
      data: _data,
      length: N,
    }
  }
}
