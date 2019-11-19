use crate::StaticVec;
use core::cmp::{Ordering, PartialOrd};

#[inline(always)]
pub(crate) fn distance_between<T>(self_: *const T, origin: *const T) -> usize {
  unsafe {
    assert!(0 < core::mem::size_of<T>());
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
pub fn new_from_value<T, const COUNT: usize>(value: T) -> StaticVec<T, { COUNT }>
where T: Copy {
  let mut res = StaticVec::<T, { COUNT }>::new();
  res.length = COUNT;
  for i in 0..COUNT {
    unsafe {
      res.data.get_unchecked_mut(i).write(value);
    }
  }
  res
}

#[inline]
pub(crate) fn partial_compare<T1, T2: PartialOrd<T1>>(
  self_: &[T2],
  other: &[T1],
) -> Option<Ordering>
{
  let min_length = self_.len().min(other.len());
  unsafe {
    let left = self_.get_unchecked(0..min_length);
    let right = other.get_unchecked(0..min_length);
    for i in 0..min_length {
      match left.get_unchecked(i).partial_cmp(right.get_unchecked(i)) {
        Some(Ordering::Equal) => (),
        non_eq => return non_eq,
      }
    }
  }
  self_.len().partial_cmp(&other.len())
}
