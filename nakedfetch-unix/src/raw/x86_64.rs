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
            "syscall",
            inlateout("rax") n => ret,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
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
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") arg1,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
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
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
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
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
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
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
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
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8")  arg5,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
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
            "syscall",
            inlateout("rax") n => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8")  arg5,
            in("r9")  arg6,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack, preserves_flags)
        );
    }
    ret
}
