#![doc = include_str!("../README.md")]

#![feature(ptr_metadata)]
#![feature(decl_macro)]
#![feature(coerce_unsized)]
#![feature(unsize)]

mod dyn_arg;

pub use dyn_arg::*;

#[cfg(feature = "derive")]
pub use dyn_struct_derive2::DynStruct;

use std::mem::{align_of, size_of};
use std::ptr::{addr_of_mut, null_mut, Pointee};
use transmute::transmute;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynStruct<Header, Tail: ?Sized> {
    pub header: Header,
    pub tail: Tail,
}

impl<Header, Tail: ?Sized> DynStruct<Header, Tail> {
    /// Allocate a new [DynStruct] on the heap.
    #[inline]
    pub fn new(header: Header, tail: DynArg<Tail>) -> Box<Self> {
        let size = Self::size(&tail);
        let align = Self::align(&tail);
        // Metadata of struct = metadata of the unsized field
        let metadata = tail.metadata();

        // Allocate actual pointer
        let thin_ptr = if size == 0 {
            // Except we can't actually allocate 0 bytes, so we return null
            null_mut() as *mut ()
        } else {
            unsafe {
                // Actually allocate
                let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
                let thin_ptr = std::alloc::alloc(layout) as *mut ();

                // Check for allocation failure
                if thin_ptr.is_null() {
                    std::alloc::handle_alloc_error(layout)
                }

                thin_ptr
            }
        };

        // Convert to fat pointer
        let ptr: *mut Self = unsafe { transmute((thin_ptr, metadata)) };

        unsafe {
            // Get header and tail pointers
            let header_ptr = addr_of_mut!((*ptr).header);
            let tail_ptr = addr_of_mut!((*ptr).tail);

            // Write header and tail
            header_ptr.write(header);
            tail.write_into(tail_ptr);
        };


        unsafe { Box::from_raw(ptr) }
    }


    /// SAFETY: `DynStruct<Header, Tail>` and `T` must have the same exact memory layout,
    /// including fields, size, and alignment. They must also have the same pointer metadata.
    ///
    /// This function checks at compile-type the pointer metadata part. Use [more_unsafe_transmute]
    /// if that check fails for some reason.
    pub unsafe fn transmute<T: Pointee<Metadata = <Tail as Pointee>::Metadata> + ?Sized>(self: Box<Self>) -> Box<T> {
        self.more_unsafe_transmute()
    }

    /// SAFETY: `DynStruct<Header, Tail>` and `T` must have the same exact memory layout,
    /// including fields, size, and alignment. They must also have the same pointer metadata.
    ///
    /// This function *does not* check at compile-type the pointer metadata part.
    /// Make sure the metadatas are the same or you will get some confusing runtime bugs.
    pub unsafe fn more_unsafe_transmute<T: ?Sized>(self: Box<Self>) -> Box<T> {
        let ptr = Box::into_raw(self);
        let ptr: *mut T = transmute(ptr);
        Box::from_raw(ptr)
    }

    #[inline]
    fn align(tail: &DynArg<Tail>) -> usize {
        usize::max(align_of::<Header>(), tail.align())
    }

    /// Returns the total size of the `DynStruct<Header, Tail>` structure, provided the length of the
    /// tail.
    #[inline]
    fn size(tail: &DynArg<Tail>) -> usize {
        let header = size_of::<Header>();
        let tail_size = tail.size();
        let tail_align = tail.align();

        let padding = if header % tail_align == 0 {
            0
        } else {
            tail_align - header % tail_align
        };

        header + padding + tail_size
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use std::rc::Rc;
    use crate::dyn_arg;
    use super::*;

    #[test]
    fn sized_types() {
        let tail = [1u64, 2, 3, 4];
        let mixed = DynStruct::new((true, 32u16), dyn_arg!(tail));
        assert_eq!(mixed.header, (true, 32u16));
        assert_eq!(&mixed.tail, &[1, 2, 3, 4]);
    }

    #[test]
    fn unsized_types() {
        let tail = [1u64, 2, 3, 4];
        let mixed = DynStruct::new((true, 32u16), dyn_arg!(tail) as DynArg<[u64]>);
        assert_eq!(mixed.header, (true, 32u16));
        assert_eq!(&mixed.tail, &[1, 2, 3, 4]);
    }

    #[test]
    fn zero_sized_types() {
        let tail = [(), ()];
        let zero = DynStruct::new((), dyn_arg!(tail));
        assert_eq!(zero.header, ());
        assert_eq!(&zero.tail, &[(), ()]);
    }

    #[test]
    fn non_copy_non_slice_types() {
        let tail = Rc::new(42) as Rc<dyn Display>;
        let tail_weak = Rc::downgrade(&tail);
        let mixed = DynStruct::new(41, dyn_arg!(tail));
        assert_eq!(mixed.header, 41);
        assert_eq!(format!("{}", mixed.tail), "42");

        // tail is still active
        assert!(tail_weak.upgrade().is_some());

        // Won't compile
        // let mixed = DynStruct::new(41, dyn_arg!(tail));

        drop(mixed); // drops tail
        assert!(tail_weak.upgrade().is_none());
    }

    #[repr(C)]
    struct SomeStruct {
        foo: bool,
        bar: usize,
        baz: [u32]
    }

    #[test]
    fn transmute() {
        let tail = [1u32, 2, 3, 4];
        let mixed = DynStruct::new((false, 50usize), dyn_arg!(tail) as DynArg<[u32]>);
        assert_eq!(mixed.header, (false, 50usize));
        assert_eq!(&mixed.tail, &[1u32, 2, 3, 4]);

        let mixed = unsafe {  mixed.transmute::<SomeStruct>() };
        assert_eq!(mixed.foo, false);
        assert_eq!(mixed.bar, 50);
        assert_eq!(&mixed.baz, &[1u32, 2, 3, 4]);

        // This is a compiler error:
        // let tail = [1u32, 2, 3, 4];
        // let mixed2 = DynStruct::new((false, 50usize), dyn_arg!(tail)); // (no coerce unsized)
        // let mixed2 = unsafe { mixed2.transmute::<SomeStruct>() }; // metadata is the wrong type
    }
}

