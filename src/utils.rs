use crate::StaticVec;

#[inline(always)]
pub(crate) fn distance_between<T>(self_: *const T, origin: *const T) -> usize {
  assert!(0 < core::intrinsics::size_of::<T>());
  unsafe {
    core::intrinsics::exact_div(
      (self_ as usize).wrapping_sub(origin as usize),
      core::intrinsics::size_of::<T>(),
    )
  }
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

#[inline(always)]
pub fn new_from_value<T, const COUNT: usize>(value: T) -> StaticVec<T, {COUNT}>
where T: Copy {
  let mut res = StaticVec::<T, {COUNT}>::new();
  res.length = COUNT;
  for i in 0..COUNT {
    unsafe {
      res.data.get_unchecked_mut(i).write(value);
    }
  }
  res
}
