use core::{
    alloc::{GlobalAlloc, Layout}, 
    ffi::c_void, 
    sync::atomic::{
        AtomicUsize, 
        AtomicU64,
        Ordering::{Relaxed, Release}
    }
};

use crate::{
    windows::link::{GetProcessHeap, HeapAlloc, HeapFree, HeapReAlloc},
    sync::OnceLock
};

const HEAP_ZERO_MEMORY: u32 = 0x08;

static HEAP_HANDLE      : OnceLock<usize> = OnceLock::new();

static ALLOC_COUNTER    : AtomicUsize     = AtomicUsize::new(0);
static REALLOC_COUNTER  : AtomicUsize     = AtomicUsize::new(0);
static DEALLOC_COUNTER  : AtomicUsize     = AtomicUsize::new(0);

static ALLOCATED_TOTAL  : AtomicU64       = AtomicU64::new(0);
static DEALLOCATED_TOTAL: AtomicU64       = AtomicU64::new(0);

static CURRENT_ALLOCATED: AtomicU64       = AtomicU64::new(0);
static MAX_IN_RUNTIME   : AtomicU64       = AtomicU64::new(0);

fn get_heap_handle() -> *mut c_void {
    // SAFETY: The `GetProcessHeap` function takes no arguments and
    // is guaranteed to return a valid handle
    let ptr = HEAP_HANDLE.get_or_init(|| 
        unsafe { GetProcessHeap() as usize }
    );
    *ptr as *mut c_void
}

fn allocated(size: usize) {
    ALLOCATED_TOTAL.fetch_add(size as u64, Relaxed);
    CURRENT_ALLOCATED.fetch_add(size as u64, Relaxed);
    let cur = CURRENT_ALLOCATED.load(Relaxed);
    if cur > MAX_IN_RUNTIME.load(Relaxed) {
        MAX_IN_RUNTIME.store(cur, Release);
    }
}

fn deallocated(size: usize) {
    DEALLOCATED_TOTAL.fetch_add(size as u64, Relaxed);
    CURRENT_ALLOCATED.fetch_sub(size as u64, Relaxed);
}

pub struct Allocator;

// SAFETY: All unsafe code has SAFETY comments
unsafe impl GlobalAlloc for Allocator {
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
        ALLOC_COUNTER.fetch_add(1, Relaxed);
        allocated(layout.size());
        
        ptr.cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
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
        DEALLOC_COUNTER.fetch_add(1, Relaxed);
        deallocated(layout.size());
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
        ALLOC_COUNTER.fetch_add(1, Relaxed);
        allocated(layout.size());
        ptr.cast::<u8>()
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let handle = get_heap_handle();

        // SAFETY: ...
        let new_ptr = unsafe {
            HeapReAlloc(handle, 0, ptr.cast(), new_size)
        };
        assert!(!new_ptr.is_null(), "`HeapReAlloc` error!");

        REALLOC_COUNTER.fetch_add(1, Relaxed);

        let old_size = layout.size();
        if new_size > old_size {
            allocated(new_size - old_size);
        } else {
            deallocated(old_size - new_size);
        }

        new_ptr.cast::<u8>()
    }
}

pub struct AllocationReport {
    pub alloc: usize,
    pub realloc: usize,
    pub dealloc: usize,
    pub alloc_total: u64,
    pub dealloc_total: u64,
    pub max_in_runtime: u64
}

impl AllocationReport {
    pub fn get() -> Self {
        let alloc = ALLOC_COUNTER.load(Relaxed);
        let realloc = REALLOC_COUNTER.load(Relaxed);
        let dealloc = DEALLOC_COUNTER.load(Relaxed);

        let alloc_total = ALLOCATED_TOTAL.load(Relaxed);
        let dealloc_total = DEALLOCATED_TOTAL.load(Relaxed);

        let max_in_runtime = MAX_IN_RUNTIME.load(Relaxed);

        Self { alloc, realloc, dealloc, alloc_total, dealloc_total, max_in_runtime }
    }
}