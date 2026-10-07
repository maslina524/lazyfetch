use core::alloc::{GlobalAlloc, Layout};

use crate::imp::libc::{malloc, calloc, free, realloc};

pub struct AllocatorInner;

// SAFETY: All unsafe code has SAFETY comments
unsafe impl GlobalAlloc for AllocatorInner {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: if the OS fails to allocate memory, it
        // returns NULL which is checked in the block
        let ptr = unsafe {
            malloc(layout.size())
        };
        debug_assert!(!ptr.is_null(), "Failed to call malloc!");
        ptr.cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // SAFETY: According to the documentation,
        // the pointer passed to `free` may be NULL (check `man 3 free``);
        unsafe { free(ptr.cast()) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The `HeapAlloc` function always
        // receives a valid handle from `GetProcessHeap`;
        // if the OS fails to allocate memory, it
        // returns NULL which is checked in the block
        let ptr = unsafe {
            calloc(
                layout.size(), 
                1
            )
        };
        debug_assert!(!ptr.is_null(), "Failed to call calloc!");
        ptr.cast::<u8>()
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: When passing a null pointer, 
        // the function will behave like `HeapAlloc`; 
        // the returned pointer is checked
        let new_ptr = unsafe {
            realloc(
                ptr.cast(),
                new_size
            )
        };
        debug_assert!(!new_ptr.is_null(), "Failed to call realloc!");
        new_ptr.cast::<u8>()
    }
}