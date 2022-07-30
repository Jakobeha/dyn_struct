//! Wrapper for dynamically-sized types we want to pass as arguments to a function.
//! Since they have dynamic size we can't pass them directly, instead we pass a fat pointer.
//! However, we need to ensure that they are inaccessible after passing them
//! and that they don't get dropped twice. This is what [DynArg] is for.

use std::marker::Unsize;
use std::mem::{align_of_val, size_of_val};
use std::ops::CoerceUnsized;
use std::ptr;
use std::ptr::{metadata, Pointee};

/// A pointer to data which was forgotten (not dropped but no active references).
/// This is a stand-in replacement for a dynamically-sized argument:
/// your function consumes this value and copy out the data,
/// instead of consuming the unsized argument directly.
pub struct DynArg<T: ?Sized>(*const T);

impl<T: ?Sized> DynArg<T> {
    /// SAFETY: You must never use `x` again after you pass it to this function,
    /// you must ensure that `x` is never dropped,
    /// and you must ensure that `x` is not deallocated until after this `DynArg` is.
    pub unsafe fn from_raw(x: *const T) -> Self {
        DynArg(x)
    }

    pub fn as_ptr(&self) -> *const T {
        self.0
    }

    pub fn size(&self) -> usize {
        size_of_val(unsafe { &*self.0 })
    }

    pub fn align(&self) -> usize {
        align_of_val(unsafe { &*self.0 })
    }

    pub fn metadata(&self) -> <T as Pointee>::Metadata {
        metadata(self.0)
    }

    pub unsafe fn write_into(self, dst: *mut T) {
        ptr::copy_nonoverlapping(self.0 as *const u8, dst as *mut u8, self.size());
    }
}

impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<DynArg<U>> for DynArg<T> {}


/// Safe way to construct [DynArg] from a stack value it consumes the input
pub macro dyn_arg($($input:tt)*) { {
    let val = $($input)*;
    let result = unsafe { DynArg::from_raw(&val) };
    ::std::mem::forget(val);
    result
} }