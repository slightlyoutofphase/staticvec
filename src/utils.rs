#[inline(always)]
pub(crate) fn distance_between<T>(_self: *const T, origin: *const T) -> usize {
  let type_size: usize = std::mem::size_of::<T>();
  assert!(0 < type_size && type_size <= std::usize::MAX);
  let distance: usize = (_self as usize).wrapping_sub(origin as usize);
  unsafe { std::intrinsics::exact_div(distance, type_size) }
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
