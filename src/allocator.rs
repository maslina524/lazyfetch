use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{
        AtomicUsize, 
        AtomicU64,
        Ordering::{Relaxed, Release}
    }
};

use crate::imp::allocator::AllocatorInner;

const HEAP_ZERO_MEMORY  : u32 = 0x08;

static ALLOC_COUNTER    : AtomicUsize = AtomicUsize::new(0);
static REALLOC_COUNTER  : AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNTER  : AtomicUsize = AtomicUsize::new(0);

static ALLOCATED_TOTAL  : AtomicU64   = AtomicU64::new(0);
static DEALLOCATED_TOTAL: AtomicU64   = AtomicU64::new(0);

static CURRENT_ALLOCATED: AtomicU64   = AtomicU64::new(0);
static MAX_IN_RUNTIME   : AtomicU64   = AtomicU64::new(0);

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
        // SAFETY: Call in unsafe fn; see the comment in `AllocatorInner`
        let ptr = unsafe { 
            AllocatorInner::alloc(&AllocatorInner, layout) 
        };

        ALLOC_COUNTER.fetch_add(1, Relaxed);
        allocated(layout.size());
        ptr.cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: Call in unsafe fn; see the comment in `AllocatorInner`
        unsafe { 
            AllocatorInner::dealloc(&AllocatorInner, ptr, layout);
        }

        DEALLOC_COUNTER.fetch_add(1, Relaxed);
        deallocated(layout.size());
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Call in unsafe fn; see the comment in `AllocatorInner`
        let ptr = unsafe { 
            AllocatorInner::alloc_zeroed(&AllocatorInner, layout) 
        };

        ALLOC_COUNTER.fetch_add(1, Relaxed);
        allocated(layout.size());
        ptr.cast::<u8>()
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: Call in unsafe fn; see the comment in `AllocatorInner`
        let new_ptr = unsafe { 
            AllocatorInner::realloc(&AllocatorInner, ptr, layout, new_size) 
        };

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