use core::ffi::c_void;

use crate::api::{
    alloc::page::{BLOCK_SIZE, DATA_SIZE, NUMBER_OF_BLOCKS, PAGE_SIZE, Page},
    mem::munmap,
};

pub fn defalloc(ptr: *mut c_void, size: usize) -> bool {
    let ptr_usize = ptr as usize;
    let mut page_ptr = Page::get_first();
    // SAFETY: ...
    let mut page = unsafe { page_ptr.as_mut() };
    loop {
        let data_usize = page.data_addr() as usize;
        if (data_usize..data_usize + DATA_SIZE).contains(&ptr_usize) {
            let local = ptr_usize - page.data_addr() as usize;
            if !local.is_multiple_of(BLOCK_SIZE) {
                return false;
            }
            let idx = local / BLOCK_SIZE;
            let blocks = size.div_ceil(BLOCK_SIZE);
            if idx + blocks > NUMBER_OF_BLOCKS {
                return false;
            }
            let mut_bitmask = page.bitmask_mut();
            for i in 0..blocks {
                mut_bitmask.unset(idx + i);
            }
            return true;
        }

        match page.next() {
            Some(p) => {
                page_ptr = p;
                // SAFETY: The pointer is guaranteed to store a Page
                page = unsafe { page_ptr.as_mut() };
            }
            None => {
                return big_defalloc(ptr);
            }
        }
    }
}

fn big_defalloc(ptr: *mut c_void) -> bool {
    let ptr_usize = ptr as usize;
    if ptr_usize < PAGE_SIZE {
        return false;
    }
    let base = (ptr_usize - PAGE_SIZE) as *mut c_void;
    // SAFETY: Boundaries are checked, safe
    let total = unsafe { *base.cast() };
    if total < 2 * PAGE_SIZE || total % PAGE_SIZE != 0 {
        return false;
    }
    let _ = munmap(base, total);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::alloc::{
        falloc,
        init::{falloc_destroy, falloc_init},
    };

    #[test]
    fn defalloc_test() {
        falloc_init();

        let size = const { size_of::<u32>() };

        let ptr = falloc(size).expect("Failed to falloc");
        defalloc(ptr, size);
        let new_ptr = falloc(size).expect("Failed to falloc");

        falloc_destroy();

        assert_eq!(new_ptr, ptr);
    }
}
