use core::{ptr, sync::atomic::Ordering::Relaxed};

use crate::api::alloc::page::{FIRST_PAGE, Page, alloc_new_page};

/// # Panics
///
/// Panics when attempting re-initialization,
/// this will never happen if you don't call this function in the main code
pub fn falloc_init() {
    let ptr = FIRST_PAGE.load(Relaxed);
    assert!(ptr.is_null(), "Attempted to initialize Falloc twice");

    let page = alloc_new_page();
    FIRST_PAGE.store(page.cast(), Relaxed);
}

/// # Panics
///
/// Panics when attempting re-destruction,
/// this will never happen if you don't call this function in the main code
pub fn falloc_destroy() {
    let page_ptr = FIRST_PAGE.load(Relaxed);
    assert!(!page_ptr.is_null(), "Attempted to destroy Falloc twice");
    Page::destroy(page_ptr);
    FIRST_PAGE.store(ptr::null_mut(), Relaxed);
}
