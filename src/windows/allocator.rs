use core::{
    alloc::{GlobalAlloc, Layout}, 
    ffi::c_void
};

use crate::{
    windows::link::{GetProcessHeap, HeapAlloc, HeapFree, HeapReAlloc},
    sync::OnceLock
};

const HEAP_ZERO_MEMORY: u32 = 0x08;

static HEAP_HANDLE    : OnceLock<usize> = OnceLock::new();

fn get_heap_handle() -> *mut c_void {
    // SAFETY: The `GetProcessHeap` function takes no arguments and
    // is guaranteed to return a valid handle
    let ptr = HEAP_HANDLE.get_or_init(|| 
        unsafe { GetProcessHeap() as usize }
    );
    *ptr as *mut c_void
}

pub struct AllocatorInner;

// SAFETY: All unsafe code has SAFETY comments
unsafe impl GlobalAlloc for AllocatorInner {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let handle = get_heap_handle();

        // SAFETY: The `HeapAlloc` function always
        // receives a valid handle from `GetProcessHeap`;
        // if the OS fails to allocate memory, it
        // returns NULL which is checked in the block
        let ptr = unsafe {
            HeapAlloc(
                handle, 
                0, 
                layout.size()
            )
        };
        assert!(!ptr.is_null(), "`HeapAlloc` error!");
        ptr.cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let handle = get_heap_handle();

        // SAFETY: According to the documentation,
        // the pointer passed to `HeapFree` may be NULL;
        // if the OS fails to free the memory, it
        // returns FALSE which is checked in the same block
        let ret = unsafe {
            HeapFree(
                handle, 
                0, 
                ptr.cast::<c_void>()
            )
        };
        assert!(ret != 0, "`HeapFree` error!");
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let handle = get_heap_handle();

        // SAFETY: The `HeapAlloc` function always
        // receives a valid handle from `GetProcessHeap`;
        // if the OS fails to allocate memory, it
        // returns NULL which is checked in the block
        let ptr = unsafe {
            HeapAlloc(
                handle, 
                HEAP_ZERO_MEMORY, 
                layout.size()
            )
        };
        assert!(!ptr.is_null(), "`HeapAlloc` error!");
        ptr.cast::<u8>()
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        let handle = get_heap_handle();

        // SAFETY: ...
        let new_ptr = unsafe {
            HeapReAlloc(handle, 0, ptr.cast(), new_size)
        };
        assert!(!new_ptr.is_null(), "`HeapReAlloc` error!");
        new_ptr.cast::<u8>()
    }
}