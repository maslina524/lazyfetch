#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![warn(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "There's no point in them, it will make the code cleaner"
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::cargo_common_metadata,
    reason = "useless"
)]

pub mod errno;
pub mod raw;
pub mod table;

pub mod io;

#[macro_export]
macro_rules! syscall {
    ($nr:expr) => {{ $crate::raw::syscall0($nr as usize) }};
    ($nr:expr, $a1:expr) => {{ $crate::raw::syscall1($nr as usize, $a1 as usize) }};
    ($nr:expr, $a1:expr, $a2:expr) => {{ $crate::raw::syscall2($nr as usize, $a1 as usize, $a2 as usize) }};
    ($nr:expr, $a1:expr, $a2:expr, $a3:expr) => {{ $crate::raw::syscall3($nr as usize, $a1 as usize, $a2 as usize, $a3 as usize) }};
    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {{
        $crate::raw::syscall4(
            $nr as usize,
            $a1 as usize,
            $a2 as usize,
            $a3 as usize,
            $a4 as usize,
        )
    }};
    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {{
        $crate::raw::syscall5(
            $nr as usize,
            $a1 as usize,
            $a2 as usize,
            $a3 as usize,
            $a4 as usize,
            $a5 as usize,
        )
    }};
    ($nr:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {{
        $crate::raw::syscall6(
            $nr as usize,
            $a1 as usize,
            $a2 as usize,
            $a3 as usize,
            $a4 as usize,
            $a5 as usize,
            $a6 as usize,
        )
    }};
}
