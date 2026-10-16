use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

use unixlib::api::alloc::{defalloc, falloc};

pub struct AllocatorInner;

// SAFETY: All unsafe code has SAFETY comments
unsafe impl GlobalAlloc for AllocatorInner {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // If the OS fails to allocate memory, it
        // returns NULL which is checked in the block
        let ptr = falloc(layout.size()).expect("Failed to call falloc!");
        ptr.cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ret = defalloc(ptr.cast(), layout.size());
        assert!(ret, "Failed to call defalloc!");
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // If the OS fails to allocate memory, it
        // returns NULL which is checked in the block
        let ptr = falloc(layout.size()).expect("Failed to call falloc!");
        // SAFETY: We write only to the allocated bytes
        unsafe {
            ptr::write_bytes(ptr, 0, layout.size());
        }
        ptr.cast::<u8>()
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size == 0 {
            // SAFETY: Deallocate valid bytes
            unsafe { self.dealloc(ptr, layout) };
            return ptr::null_mut();
        }

        let new_ptr = falloc(new_size)
            .expect("Failed to call falloc!")
            .cast::<u8>();

        let copy_size = core::cmp::min(layout.size(), new_size);
        // SAFETY: new_ptr выделен под new_size, старый ptr — под layout.size().
        // copy_size <= обоих размеров.
        unsafe {
            ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
            self.dealloc(ptr, layout);
        }

        new_ptr
    }
}
