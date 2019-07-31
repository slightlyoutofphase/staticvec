#[inline(always)]
pub(crate) fn reverse_copy<T>(first: *const T, mut last: *const T, mut result: *mut T)
where T: Copy {
  while first != last {
    unsafe {
      last = last.sub(1);
      *result = *last;
      result = result.add(1);
    }
  }
}

#[inline(always)]
pub(crate) fn distance_between<T>(_self: *const T, origin: *const T) -> usize {
  let type_size = std::mem::size_of::<T>();
  assert!(0 < type_size && type_size <= std::usize::MAX);
  let distance = (_self as usize).wrapping_sub(origin as usize);
  unsafe { std::intrinsics::exact_div(distance, type_size) }
}
