#[macro_export]
macro_rules! todo_or_default {
    ($msg:literal, $default:expr) => {{
        #[deprecated(note = $msg)]
        const fn _warn() {}
        _warn();
        $default
    }};
}

#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {{
        $crate::print!("\x1b[1;{}mInfo\x1b[0m: ", $crate::color::FG_BLUE);
        $crate::println!($($args)*);
    }};
}

#[macro_export]
macro_rules! warning {
    ($($args:tt)*) => {{
        #[cfg(debug_assertions)]
        $crate::eprint!("\x1b[1;{}mWarning\x1b[0m:{}:{}: ", $crate::color::FG_YELLOW, file!(), line!());

        #[cfg(not(debug_assertions))]
        $crate::eprint!("\x1b[1;{}mWarning\x1b[0m: ", $crate::color::FG_YELLOW);

        $crate::eprintln!($($args)*);
    }};
}

#[macro_export]
macro_rules! abort {
    ($($args:tt)*) => {{
        $crate::eprintln!("\x1b[1;{}mProgram Aborted\x1b[0m", $crate::color::FG_RED);
        panic!($($args)*);
    }};
}

#[cfg(target_os = "windows")]
#[macro_export]
macro_rules! get_fn {
    ($handle:tt, $name:expr, $typ:ident) => {{
        // SAFETY: Completely safe
        let addr = $crate::windows::link::GetProcAddress($handle, $name.as_ptr().cast())
            .unwrap_or_else(|| {
                unload($handle);
                $crate::abort!(concat!(stringify!($name), " not found in dll"));
            });
        core::mem::transmute::<ApiBaseFn, $typ>(addr)
    }};
}

#[cfg(target_family = "unix")]
#[macro_export]
macro_rules! get_fn {
    ($handle:tt, $name:expr, $typ:ident) => {{
        // SAFETY: Completely safe
        let addr = $crate::unix::libc::dlsym($handle, $name.as_ptr().cast());
        if addr.is_null() {
            unload($handle);
            $crate::abort!(concat!(stringify!($name), " not found in library"));
        }
        core::mem::transmute::<ApiBaseFn, $typ>(addr)
    }};
}

#[macro_export]
/// Copied from `cfg_if`
///
/// Source link: <https://docs.rs/cfg-if/latest/src/cfg_if/lib.rs.html#1-212>
macro_rules! cfg_if {
    (
        if #[cfg( $($i_meta:tt)+ )] { $( $i_tokens:tt )* }
        $(
            else if #[cfg( $($ei_meta:tt)+ )] { $( $ei_tokens:tt )* }
        )*
        $(
            else { $( $e_tokens:tt )* }
        )?
    ) => {
        $crate::cfg_if! {
            @__items () ;
            (( $($i_meta)+ ) ( $( $i_tokens )* )),
            $(
                (( $($ei_meta)+ ) ( $( $ei_tokens )* )),
            )*
            $(
                (() ( $( $e_tokens )* )),
            )?
        }
    };

    (@__items ( $( ($($_:tt)*) , )* ) ; ) => {};
    (
        @__items ( $( ($($no:tt)+) , )* ) ;
        (( $( $($yes:tt)+ )? ) ( $( $tokens:tt )* )),
        $( $rest:tt , )*
    ) => {
        #[cfg(all(
            $( $($yes)+ , )?
            not(any( $( $($no)+ ),* ))
        ))]

        $crate::cfg_if! { @__temp_group $( $tokens )* }

        $crate::cfg_if! {
            @__items ( $( ($($no)+) , )* $( ($($yes)+) , )? ) ;
            $( $rest , )*
        }
    };

    (@__temp_group $( $tokens:tt )* ) => {
        $( $tokens )*
    };
}

#[macro_export]
macro_rules! format {
    ($($tt:tt)*) => {{
        let mut string = alloc::string::String::with_capacity(16);
        let mut formatter = $crate::formats::StringFormatter::new(&mut string);
        let _ = formatter.write_fmt(format_args!($($tt)*));
        string
    }};
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}:{}]", core::file!(), core::line!(), core::column!())
    };
    ($flag:ident) => {
        if $crate::imp::env::contains_in_dbg_args(stringify!($flag)) {
            $crate::dbg!();
        }
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    column!(),
                    stringify!($val),
                    &&tmp as &dyn core::fmt::Debug,
                );
                tmp
            }
        }
    };
    ($val:expr $(,)?, $flag:ident) => {
        if $crate::imp::env::contains_in_dbg_args(stringify!($flag)) {
            $crate::dbg!($val)
        } else {
            $val
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+)
    };
    ($($val:expr),+ $(,)?, $flag:ident) => {
        if $crate::imp::env::contains_in_dbg_args(stringify!($flag)) {
            ($($crate::dbg!($val)),+)
        } else {
            ($($val),+)
        }
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! dbg {
    () => {};
    ($flag:ident) => {};
    ($val:expr $(,)?) => {};
    ($val:expr $(,)?, $flag:ident) => {};
    ($($val:expr),+ $(,)?) => {};
    ($($val:expr),+ $(,)?, $flag:ident) => {};
}

#[macro_export]
macro_rules! w {
    ($s:literal) => {{
        const fn to_utf16<const N: usize>(s: &str) -> [u16; N] {
            let bytes = s.as_bytes();
            let mut out = [0u16; N];
            let mut i = 0;
            let mut o = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if b < 0x80 {
                    out[o] = b as u16;
                    i += 1;
                }
                o += 1;
            }
            out[o] = 0;
            out
        }
        const fn count(s: &str) -> usize {
            s.as_bytes().len() + 1
        }
        to_utf16::<{ count($s) }>($s)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn warning_test() {
        warning!("Example warn");
        warning!("Formatted msg: 2 + 2 = {}", 2 + 2);
    }
}
