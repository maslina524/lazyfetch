use core::{
    ffi::c_void, ptr::{self, NonNull}, sync::atomic::{AtomicPtr, Ordering::Relaxed},
};

use crate::api::mem::{flags, mmap, munmap, prot};

pub const PAGE_SIZE: usize = 64 * 1024;
pub const BLOCK_SIZE: usize = 128;
pub const DATA_OFFSET: usize = BLOCK_SIZE;
pub const DATA_SIZE: usize = PAGE_SIZE - DATA_OFFSET;
pub const NUMBER_OF_BLOCKS: usize = DATA_SIZE / BLOCK_SIZE;
pub const NUMBER_OF_BLOCKS_BYTES: usize = (NUMBER_OF_BLOCKS + 7) >> 3;

const _: () = assert!(size_of::<Page>() <= BLOCK_SIZE);

#[repr(transparent)]
pub struct BitMask<const BYTES: usize> {
    inner: [u8; BYTES],
}

impl<const BYTES: usize> BitMask<BYTES> {
    pub const fn new() -> Self {
        Self {
            inner: [0u8; BYTES],
        }
    }

    #[inline]
    pub const fn get(&self, pos: usize) -> bool {
        let byte = pos >> 3;
        let bit = pos & 7;
        (self.inner[byte] >> bit) & 1 == 1
    }

    #[inline]
    pub const fn set(&mut self, pos: usize) {
        let byte = pos >> 3;
        let bit = pos & 7;
        let mask = 1u8 << bit;
        self.inner[byte] |= mask;
    }

    #[inline]
    pub const fn unset(&mut self, pos: usize) {
        let byte = pos >> 3;
        let bit = pos & 7;
        let mask = 1u8 << bit;
        self.inner[byte] &= !mask;
    }
}

impl<const BYTES: usize> Default for BitMask<BYTES> {
    fn default() -> Self {
        Self::new()
    }
}

pub static FIRST_PAGE: AtomicPtr<Page> = AtomicPtr::new(ptr::null_mut());

#[derive(Default)]
pub struct Page {
    blocks_mask: BitMask<NUMBER_OF_BLOCKS_BYTES>,
    mmap_addr: AtomicPtr<c_void>,
    next: AtomicPtr<Self>,
}

impl Page {
    pub const fn new(mmap_addr: *mut c_void) -> Self {
        Self {
            blocks_mask: BitMask::new(),
            mmap_addr: AtomicPtr::new(mmap_addr),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn get_first() -> NonNull<Self> {
        let ptr = FIRST_PAGE.load(Relaxed);
        NonNull::new(ptr)
            .expect("Falloc not initialized")
    }

    pub fn destroy(this: *mut Self) {
        if this.is_null() {
            return;
        }
        // SAFETY: Checked for a non-null pointer
        let this = unsafe { &*this };

        let next_page_ptr = this.next.load(Relaxed);
        if !next_page_ptr.is_null() {
            Self::destroy(next_page_ptr);
        }

        let mmap_addr = this.mmap_addr.load(Relaxed);
        let _ = munmap(mmap_addr, PAGE_SIZE);
    }

    pub const fn bitmask(&self) -> &BitMask<NUMBER_OF_BLOCKS_BYTES> {
        &self.blocks_mask
    }

    pub const fn bitmask_mut(&mut self) -> &mut BitMask<NUMBER_OF_BLOCKS_BYTES> {
        &mut self.blocks_mask
    }

    pub fn mmap_addr(&self) -> *mut c_void {
        self.mmap_addr.load(Relaxed)
    }

    pub fn data_addr(&self) -> *mut c_void {
        // SAFETY: Moving only within allocated memory
        unsafe { self.mmap_addr().add(DATA_OFFSET) }
    }

    pub fn block_addr(&self, n: usize) -> Option<*mut c_void> {
        if n >= NUMBER_OF_BLOCKS {
            return None
        }

        // SAFETY: We check the mmap boundary, safe
        let addr = unsafe { self.data_addr().add(n * BLOCK_SIZE) };
        Some(addr)
    }

    pub fn next(&self) -> Option<NonNull<Self>> {
        let ptr = self.next.load(Relaxed);
        NonNull::new(ptr)
    }

    pub fn set_next(&self, next: *mut Self) {
        let ptr = self.next.load(Relaxed);
        assert!(!ptr.is_null(), "Attempt to overwrite info about the next page");

        self.next.store(next, Relaxed);
    }
}

pub fn alloc_new_page() -> *mut Page {
    let mmap = mmap(
        ptr::null(),
        PAGE_SIZE,
        prot::READ | prot::WRITE,
        flags::PRIVATE | flags::ANONYMOUS,
        -1,
        0,
    )
    .expect("Failed to create page for Falloc");

    let page = Page::new(mmap);
    // SAFETY: The pointer is valid, safe
    unsafe {
        ptr::write(mmap.cast(), page);
    }

    mmap.cast()
}