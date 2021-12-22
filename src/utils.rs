use core::cmp::{Ordering, PartialOrd};
use core::intrinsics::{assume, ptr_offset_from};
use core::mem::{/* align_of, */ size_of, MaybeUninit};

use crate::StaticVec;

/// An internal function for calculating pointer offsets as usizes, while accounting
/// directly for possible ZSTs. This is used specifically in the iterator implementations.
#[inline(always)]
pub(crate) const fn distance_between<T>(dest: *const T, origin: *const T) -> usize {
  // Safety: this function is used strictly with linear inputs
  // where `dest` is known to come after `origin`.
  match size_of::<T>() {
    0 => unsafe { ptr_offset_from(dest as *const u8, origin as *const u8) as usize },
    _ => unsafe { ptr_offset_from(dest, origin) as usize },
  }
}

/// A `const fn` compatible `min` function specifically for usizes. Not being generic allows it to
/// actually work in the const contexts we need it to.
#[inline(always)]
pub(crate) const fn const_min(lhs: usize, rhs: usize) -> usize {
  if lhs < rhs {
    lhs
  } else {
    rhs
  }
}

/// A simple reversal function that returns a new array, called in
/// [`StaticVec::reversed`](crate::StaticVec::reversed).
#[inline]
pub(crate) const fn reverse_copy<T, const N: usize>(
  length: usize,
  this: &MaybeUninit<[T; N]>,
) -> MaybeUninit<[T; N]>
where
  T: Copy,
{
  let mut i = length;
  let src = StaticVec::first_ptr(this);
  let mut res = StaticVec::new_data_uninit();
  let mut dest = StaticVec::first_ptr_mut(&mut res);
  // We know these are valid pointers based on how and where this
  // function is called from, so these are safe hints to give to the
  // optimizer.
  unsafe {
    assume(!src.is_null());
    // Curiously, the explicit typecast to `*mut T` on the next line
    // is necessary to get it to compile. Without the typecast, `rustc` can't figure out
    // what the type is supposed to be for some reason.
    assume(!(dest as *mut T).is_null());
  }
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
          // Can't use `first_ptr_mut` here as the type inference doesn't work
          // in this context for some reason.
          (data.as_mut_ptr() as *mut T).add(i).write(value);
        }
        data
      }
    },
    length: COUNT,
  }
}

/// An internal convenience function for incrementing mutable ZST pointers by usize offsets.
#[inline(always)]
pub(crate) const fn zst_ptr_add_mut<T>(ptr: *mut T, offset: usize) -> *mut T {
  unsafe { (ptr as *mut u8).offset(count as isize) as *mut T }
}

/// An internal convenience function for incrementing immutable ZST pointers by usize offsets.
#[inline(always)]
pub(crate) const fn zst_ptr_add<T>(ptr: *const T, offset: usize) -> *const T {
  unsafe { (ptr as *const u8).offset(count as isize) as *const T }
}

/// A version of the default `partial_cmp` implementation with a more flexible function signature.
#[inline]
pub(crate) fn partial_compare<T1, T2: PartialOrd<T1>>(
  this: &[T2],
  other: &[T1],
) -> Option<Ordering> {
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

/// A simple quicksort function for internal use, called in
/// ['quicksorted_unstable`](crate::StaticVec::quicksorted_unstable).
#[inline]
pub(crate) /*const*/ fn quicksort_internal<T: Copy + PartialOrd>(
  values: *mut T,
  mut low: isize,
  mut high: isize,
) {
  // We call this function from exactly one place where `low` and `high` are known to be within an
  // appropriate range before getting passed into it, so there's no need to check them again here.
  // We also know that `values` will never be null, so we can safely give an optimizer hint here.
  unsafe { assume(!values.is_null()) };
  loop {
    let mut i = low;
    let mut j = high;
    unsafe {
      let p = *values.offset(low + ((high - low) >> 1));
      loop {
        while *values.offset(i) < p {
          i += 1;
        }
        while *values.offset(j) > p {
          j -= 1;
        }
        if i <= j {
          if i != j {
            let q = *values.offset(i);
            *values.offset(i) = *values.offset(j);
            *values.offset(j) = q;
          }
          i += 1;
          j -= 1;
        }
        if i > j {
          break;
        }
      }
    }
    if j - low < high - i {
      if low < j {
        quicksort_internal(values, low, j);
      }
      low = i;
    } else {
      if i < high {
        quicksort_internal(values, i, high)
      }
      high = j;
    }
    if low >= high {
      break;
    }
  }
}

/*
/// A local (identically written) `const fn` version of `intrinsics::is_aligned_and_not_null`.
#[inline(always)]
pub(crate) const fn is_aligned_and_not_null<T>(ptr: *const T) -> bool {
  // This does not compile currently in certain const contexts because of the `ptr as usize` cast,
  // even though the results of the below slice functions are always valid and properly usable
  // from regular code even when declared as `static`.
  unsafe { !ptr.is_null() && ptr as usize % align_of::<T>() == 0 }
}
*/

/// A local (identically written) `const fn` version of `slice::from_raw_parts`.
#[inline(always)]
pub(crate) const fn slice_from_raw_parts<'a, T>(data: *const T, length: usize) -> &'a [T] {
  debug_assert!(
    /*
    is_aligned_and_not_null(data),
    "Attempted to create an unaligned or null slice!"
    */
    // See comment starting at line 165 for more info about what's going on here. Note that the
    // alignment check is not actually a concern for our use case anyways, since we only call this
    // function with known-valid pointers to initialized elements of a StaticVec's internal array.
    !data.is_null(),
    "Attempted to create a null slice!"
  );
  debug_assert!(
    size_of::<T>().saturating_mul(length) <= isize::MAX as usize,
    "Attempted to create a slice covering at least half of the address space!"
  );
  unsafe { &*core::ptr::slice_from_raw_parts(data, length) }
}

/// A local (identically written) `const fn` version of `slice::from_raw_parts_mut`.
#[inline(always)]
pub(crate) const fn slice_from_raw_parts_mut<'a, T>(data: *mut T, length: usize) -> &'a mut [T] {
  debug_assert!(
    /*
    is_aligned_and_not_null(data),
    "Attempted to create an unaligned or null slice!"
    */
    // See comment starting at line 165 for more info about what's going on here. Note that the
    // alignment check is not actually a concern for our use case anyways, since we only call this
    // function with known-valid pointers to initialized elements of a StaticVec's internal array.
    !data.is_null(),
    "Attempted to create a null slice!"
  );
  debug_assert!(
    size_of::<T>().saturating_mul(length) <= isize::MAX as usize,
    "Attempted to create a slice covering at least half of the address space!"
  );
  unsafe { &mut *core::ptr::slice_from_raw_parts_mut(data, length) }
}
