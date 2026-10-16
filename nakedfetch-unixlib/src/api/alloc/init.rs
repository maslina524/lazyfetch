use core::{ptr, sync::atomic::Ordering::Relaxed};

use crate::api::{
    alloc::page::{FIRST_PAGE, PAGE_SIZE, Page},
    mem::{flags, mmap, prot},
};

/// # Panics
///
/// Panics when attempting re-initialization,
/// this will never happen if you don't call this function in the main code
pub fn falloc_init() {
    let ptr = FIRST_PAGE.load(Relaxed);
    assert!(ptr.is_null(), "Attempted to initialize Falloc twice");

    let ret = mmap(
        ptr::null(),
        PAGE_SIZE,
        prot::READ | prot::WRITE,
        flags::PRIVATE | flags::ANONYMOUS,
        -1,
        0,
    )
    .expect("Failed to create page for Falloc");

    let page_ptr = ret.cast();
    // SAFETY: The pointer is valid, safe
    unsafe {
        ptr::write(page_ptr, Page::new(ret));
    }

    FIRST_PAGE.store(page_ptr, Relaxed);
}

/// # Panics
///
/// Panics when attempting re-destruction,
/// this will never happen if you don't call this function in the main code
pub fn falloc_destroy() {
    let page_ptr = FIRST_PAGE.load(Relaxed);
    assert!(!page_ptr.is_null(), "Attempted to destroy Falloc twice");

    // SAFETY: Checked for a non-null pointer
    let page = unsafe { &*page_ptr };
    page.destroy();
}
