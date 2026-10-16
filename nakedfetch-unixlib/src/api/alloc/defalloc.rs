use core::{alloc::Layout, ffi::c_void};

use crate::api::alloc::page::{BLOCK_SIZE, DATA_SIZE, Page};

pub fn defalloc(ptr: *mut c_void, layout: Layout) -> bool {
    let ptr_usize = ptr as usize;
    let mut page_ptr = Page::get_first();
    // SAFETY: The pointer is guaranteed to store a Page
    let mut page = unsafe { page_ptr.as_mut() };
    loop {
        let data_usize = page.data_addr() as usize;
        if (data_usize..data_usize + DATA_SIZE).contains(&ptr_usize) {
            break;
        }

        page_ptr = if let Some(p) = page.next() {
            p
        } else {
            return false;
        };
        // SAFETY: The pointer is guaranteed to store a Page
        page = unsafe { page_ptr.as_mut() };
    }

    let local_ptr_in_data = ptr_usize - page.data_addr() as usize;
    if !local_ptr_in_data.is_multiple_of(BLOCK_SIZE) {
        return false;
    }

    let idx = local_ptr_in_data / BLOCK_SIZE;
    let blocks = layout.size().div_ceil(BLOCK_SIZE);
    let mut_bitmask = page.bitmask_mut();

    for i in 0..blocks {
        mut_bitmask.unset(idx + i);
    }

    true
}

#[cfg(test)]
mod tests {
    use core::alloc::Layout;

    use super::*;
    use crate::api::alloc::{
        falloc,
        init::{falloc_destroy, falloc_init},
    };

    #[test]
    fn defalloc_test() {
        falloc_init();

        let layout = Layout::new::<i32>();

        let ptr = falloc(layout).expect("Failed to falloc");
        defalloc(ptr, layout);
        let new_ptr = falloc(layout).expect("Failed to falloc");

        falloc_destroy();

        assert_eq!(new_ptr, ptr);
    }
}
