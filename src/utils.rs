use crate::StaticVec;
use core::cmp::{Ordering, PartialOrd};
use core::intrinsics;
use core::mem::MaybeUninit;
use core::ptr;

/// An internal convenience function to go from `*const MaybeUninit<[T; N]>` to `*const T`.
#[inline(always)]
pub(crate) fn ptr_const<T, const N: usize>(this: *const MaybeUninit<[T; N]>) -> *const T {
  this as *const T
}

/// An internal convenience function to go from `*mut MaybeUninit<[T; N]>` to `*mut T`.
#[inline(always)]
pub(crate) fn ptr_mut<T, const N: usize>(this: *mut MaybeUninit<[T; N]>) -> *mut T {
  this as *mut T
}

/// An internal function for calculating pointer offsets as usizes, while accounting
/// directly for possible ZSTs. This is used specifically in the iterator implementations.
#[inline(always)]
pub(crate) const fn distance_between<T>(dest: *const T, origin: *const T) -> usize {
  match intrinsics::size_of::<T>() {
    0 => unsafe { (dest as usize).wrapping_sub(origin as usize) },
    // Safety: this function is used strictly with linear inputs
    // where dest is known to come after origin.
    _ => unsafe { intrinsics::ptr_offset_from(dest, origin) as usize },
  }
}

/// A simple reversal function that returns a new array, called in
/// [`StaticVec::reversed`](crate::StaticVec::reversed).
#[inline]
pub(crate) fn reverse_copy<T, const N: usize>(
  length: usize,
  this: *const MaybeUninit<[T; N]>,
) -> MaybeUninit<[T; N]>
where
  T: Copy,
{
  let mut i = length;
  let src = ptr_const(this);
  let mut res: MaybeUninit<[T; N]> = MaybeUninit::uninit();
  let mut dest = ptr_mut(&mut res);
  while i > 0 {
    unsafe {
      src.add(i - 1).copy_to_nonoverlapping(dest, 1);
      dest = dest.offset(1);
      i -= 1;
    }
  }
  res
}

/// Previously this was what one of the forms of the [`staticvec!`] macro used internally. Currently
/// it's not used at all, and may be removed if I don't think of another use for it in the next
/// little while.
#[inline(always)]
pub fn new_from_value<T, const COUNT: usize>(value: T) -> StaticVec<T, COUNT>
where T: Copy {
  StaticVec {
    data: {
      unsafe {
        let mut data = StaticVec::new_data_uninit();
        for i in 0..COUNT {
          // Can't use ptr_mut() here as the type inference doesn't work
          // in this context for some reason.
          (data.as_mut_ptr() as *mut T).add(i).write(value);
        }
        data
      }
    },
    length: COUNT,
  }
}

/// A version of the default `partial_cmp` implementation with a more flexible function signature.
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

/// A local inline-always version of `slice::from_raw_parts`.
#[inline(always)]
pub(crate) fn make_const_slice<'a, T>(data: *const T, length: usize) -> &'a [T] {
  unsafe { &*ptr::slice_from_raw_parts(data, length) }
}

/// A local inline-always version of `slice::from_raw_parts_mut`.
#[inline(always)]
pub(crate) fn make_mut_slice<'a, T>(data: *mut T, length: usize) -> &'a mut [T] {
  unsafe { &mut *ptr::slice_from_raw_parts_mut(data, length) }
}
