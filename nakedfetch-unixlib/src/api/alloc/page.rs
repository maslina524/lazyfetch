use core::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicPtr, Ordering::Relaxed},
};

use crate::api::mem::munmap;

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
    pub const fn set(&mut self, pos: usize, value: bool) {
        let byte = pos >> 3;
        let bit = pos & 7;
        let mask = 1u8 << bit;
        if value {
            self.inner[byte] |= mask;
        } else {
            self.inner[byte] &= !mask;
        }
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

    pub fn destroy(&self) {
        let next_page_ptr = self.next.load(Relaxed);
        if !next_page_ptr.is_null() {
            // SAFETY: Checked for a non-null pointer
            let next_page = unsafe { &*next_page_ptr };
            next_page.destroy();
        }

        let mmap_addr = self.mmap_addr.load(Relaxed);
        let _ = munmap(mmap_addr, PAGE_SIZE);
    }
}
