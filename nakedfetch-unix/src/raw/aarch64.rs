use core::arch::asm;

/// # SAFETY
///
/// Calling a raw syscall can lead to unexpected behavior, loss of safety, etc.
#[inline]
pub unsafe fn syscall0(n: usize) -> isize {
    let mut ret: isize;
    // SAFETY: The function does not take responsibility!
    unsafe {
        asm!(
            "svc 0",
            in("x8") n,
            lateout("x0") ret,
            lateout("x16") _,
            lateout("x17") _,
            options(nostack, preserves_flags)
        )
    };
    ret
}

/// # SAFETY
///
/// Calling a raw syscall can lead to unexpected behavior, loss of safety, etc.
#[inline]
pub unsafe fn syscall1(n: usize, arg1: usize) -> isize {
    let mut ret: isize;
    // SAFETY: The function does not take responsibility!
    unsafe {
        asm!(
            "svc 0",
            in("x8") n,
            inlateout("x0") arg1 => ret,
            lateout("x16") _,
            lateout("x17") _,
            options(nostack, preserves_flags)
        )
    };
    ret
}

/// # SAFETY
///
/// Calling a raw syscall can lead to unexpected behavior, loss of safety, etc.
#[inline]
pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> isize {
    let mut ret: isize;
    // SAFETY: The function does not take responsibility!
    unsafe {
        asm!(
            "svc 0",
            in("x8") n,
            inlateout("x0") arg1 => ret,
            in("x1") arg2,
            lateout("x16") _,
            lateout("x17") _,
            options(nostack, preserves_flags)
        )
    };
    ret
}

/// # SAFETY
///
/// Calling a raw syscall can lead to unexpected behavior, loss of safety, etc.
#[inline]
pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let mut ret: isize;
    // SAFETY: The function does not take responsibility!
    unsafe {
        asm!(
            "svc 0",
            in("x8") n,
            inlateout("x0") arg1 => ret,
            in("x1") arg2,
            in("x2") arg3,
            lateout("x16") _,
            lateout("x17") _,
            options(nostack, preserves_flags)
        )
    };
    ret
}

/// # SAFETY
///
/// Calling a raw syscall can lead to unexpected behavior, loss of safety, etc.
#[inline]
pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> isize {
    let mut ret: isize;
    // SAFETY: The function does not take responsibility!
    unsafe {
        asm!(
            "svc 0",
            in("x8") n,
            inlateout("x0") arg1 => ret,
            in("x1") arg2,
            in("x2") arg3,
            in("x3") arg4,
            lateout("x16") _,
            lateout("x17") _,
            options(nostack, preserves_flags)
        )
    };
    ret
}

/// # SAFETY
///
/// Calling a raw syscall can lead to unexpected behavior, loss of safety, etc.
#[inline]
pub unsafe fn syscall5(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> isize {
    let mut ret: isize;
    // SAFETY: The function does not take responsibility!
    unsafe {
        asm!(
            "svc 0",
            in("x8") n,
            inlateout("x0") arg1 => ret,
            in("x1") arg2,
            in("x2") arg3,
            in("x3") arg4,
            in("x4") arg5,
            lateout("x16") _,
            lateout("x17") _,
            options(nostack, preserves_flags)
        )
    };
    ret
}

/// # SAFETY
///
/// Calling a raw syscall can lead to unexpected behavior, loss of safety, etc.
#[inline]
pub unsafe fn syscall6(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let mut ret: isize;
    // SAFETY: The function does not take responsibility!
    unsafe {
        asm!(
            "svc 0",
            in("x8") n,
            inlateout("x0") arg1 => ret,
            in("x1") arg2,
            in("x2") arg3,
            in("x3") arg4,
            in("x4") arg5,
            in("x5") arg6,
            lateout("x16") _,
            lateout("x17") _,
            options(nostack, preserves_flags)
        )
    };
    ret
}
