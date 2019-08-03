#[inline(always)]
pub(crate) fn distance_between<T>(self_: *const T, origin: *const T) -> usize {
  let type_size: usize = core::mem::size_of::<T>();
  assert!(0 < type_size);
  let distance: usize = (self_ as usize).wrapping_sub(origin as usize);
  unsafe { core::intrinsics::exact_div(distance, type_size) }
}

#[inline(always)]
pub(crate) fn reverse_copy<T>(first: *const T, mut last: *const T, mut result: *mut T)
where T: Copy {
  while first != last {
    unsafe {
      last = last.offset(-1);
      *result = *last;
      result = result.offset(1);
    }
  }
}
