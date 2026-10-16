use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use crate::{
    api::{alloc::page::{BLOCK_SIZE, NUMBER_OF_BLOCKS, PAGE_SIZE, Page, alloc_new_page}, mem::{flags, mmap, prot}}, errno::ErrnoResult,
};

#[allow(clippy::missing_panics_doc)]
pub fn falloc(size: usize) -> ErrnoResult<*mut c_void> {
    if size == 0 {
        return Ok(ptr::null_mut());
    }
    let blocks = size.div_ceil(BLOCK_SIZE);

    if blocks > NUMBER_OF_BLOCKS {
        let total = (size + 2 * PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
        let mmap_addr = mmap(
            ptr::null(),
            total,
            prot::READ | prot::WRITE,
            flags::PRIVATE | flags::ANONYMOUS,
            -1,
            0,
        )?;

        // SAFETY: `mmap` returns `total` valid bytes
        unsafe {
            *mmap_addr.cast() = total;
        }
        // SAFETY: Boundaries are checked, safe
        let user_ptr = unsafe { mmap_addr.cast::<u8>().add(PAGE_SIZE) };
        return Ok(user_ptr.cast());
    }

    let mut idx = None;
    let mut page_ptr = Page::get_first();
    // SAFETY: The pointer is guaranteed to store a Page
    let mut page = unsafe { page_ptr.as_mut() };
    loop {
        let bitmask = page.bitmask();
        for i in 0..=NUMBER_OF_BLOCKS - blocks {
            let mut skip = false;
            for s in i..blocks + i {
                if bitmask.get(s) {
                    skip = true;
                    break;
                }
            }
            if !skip {
                idx = Some(i);
                break;
            }
        }
        if idx.is_some() {
            break;
        }

        page_ptr = page.next().unwrap_or_else(|| {
            let new_page = alloc_new_page();
            page.set_next(new_page);
            NonNull::new(new_page).expect("alloc_new_page returned null ptr")
        });
        // SAFETY: The pointer is guaranteed to store a Page
        page = unsafe { page_ptr.as_mut() };
    }

    let idx = idx.unwrap();
    let mut_bitmask = page.bitmask_mut();
    for i in idx..blocks + idx {
        mut_bitmask.set(i);
    }

    Ok(page.block_addr(idx).unwrap())
}

#[cfg(test)]
mod tests {
    use crate::api::alloc::init::{falloc_destroy, falloc_init};

    use super::*;

    const THREE_PAGES: usize = 511 * 3;
    #[test]
    fn falloc_test() {
        falloc_init();

        for _ in 0..THREE_PAGES {
            let ret = falloc(BLOCK_SIZE)
                .expect("Falloc returned Err");

            // UB check
            #[allow(clippy::undocumented_unsafe_blocks)]
            unsafe {
                ptr::write_bytes(ret, 42, BLOCK_SIZE);
            }

            for i in 0..BLOCK_SIZE {
                #[allow(clippy::undocumented_unsafe_blocks)]
                let val = unsafe {
                    let moved_ptr = ret.cast::<u8>().add(i);
                    ptr::read(moved_ptr)
                };

                assert_eq!(val, 42);
            }
        }

        falloc_destroy();
    }
}
