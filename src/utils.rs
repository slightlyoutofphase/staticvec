use crate::StaticVec;
use core::cmp::{Ordering, PartialOrd};
use core::mem::MaybeUninit;
use core::intrinsics;

#[inline(always)]
pub(crate) fn distance_between<T>(dest: *const T, origin: *const T) -> usize {
  unsafe {
    return if intrinsics::size_of::<T>() > 0 {
      intrinsics::exact_div(
        (dest as usize).wrapping_sub(origin as usize),
        intrinsics::size_of::<T>(),
      )
    } else {
      (dest as usize).wrapping_sub(origin as usize)
    };
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
  StaticVec::<T, { COUNT }> {
    data: {
      let mut data: [MaybeUninit<T>; COUNT] = MaybeUninit::uninit().assume_init();
      for i in 0..COUNT {
        data.get_unchecked_mut(i).write(value);
      }
      data
    },
    length: COUNT,
  }
}

#[inline]
pub(crate) fn partial_compare<T1, T2: PartialOrd<T1>>(
  this: &[T2],
  other: &[T1],
) -> Option<Ordering>
{
  let min_length = this.len().min(other.len());
  unsafe {
    let left = this.get_unchecked(0..min_length);
    let right = other.get_unchecked(0..min_length);
    for i in 0..min_length {
      match left.get_unchecked(i).partial_cmp(right.get_unchecked(i)) {
        Some(Ordering::Equal) => (),
        non_eq => return non_eq,
      }
    }
  }
  this.len().partial_cmp(&other.len())
}
