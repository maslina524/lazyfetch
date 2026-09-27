use core::fmt::{Arguments, Write};

use alloc::{
    vec::Vec,
    string::String
};

use crate::{sync::Mutex, imp::io::{stdout, stderr, write}};

const STRING_BASE_CAP: usize = 64;

#[derive(Debug)]
enum OutputType {
    Stdout,
    Stderr
}

#[derive(Debug)]
struct Output {
    typ: OutputType,
    inner: String
}

impl Output {
    pub const fn stdout(inner: String) -> Self {
        Self { typ: OutputType::Stdout, inner }
    }

    pub const fn stderr(inner: String) -> Self {
        Self { typ: OutputType::Stderr, inner }
    }
}

static BUFFER: Mutex<Vec<Output>> = Mutex::new(Vec::new());

pub fn write_stdout(args: Arguments<'_>) {
    let mut guard = BUFFER.lock();
    match guard.last_mut() {
        Some(o) if matches!(o.typ, OutputType::Stdout) => {
            let _ = o.inner.write_fmt(args);
        }
        _ => {
            let mut s = String::with_capacity(STRING_BASE_CAP);
            let _ = s.write_fmt(args);
            guard.push(Output::stdout(s));
        }
    }
}

pub fn write_stderr(args: Arguments<'_>) {
    let mut guard = BUFFER.lock();
    match guard.last_mut() {
        Some(o) if matches!(o.typ, OutputType::Stderr) => {
            let _ = o.inner.write_fmt(args);
        }
        _ => {
            let mut s = String::with_capacity(STRING_BASE_CAP);
            let _ = s.write_fmt(args);
            guard.push(Output::stderr(s));
        }
    }
}

pub fn flush() -> core::fmt::Result {
    let guard = BUFFER.lock();

    // Stdout::get().write_fmt(format_args!("{:#?}", *guard))?;

    for o in &*guard {
        match o.typ {
            OutputType::Stdout => Stdout::get().write_str(&o.inner)?,
            OutputType::Stderr => Stderr::get().write_str(&o.inner)?,
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
pub struct Stdout(isize);

impl Stdout {
    pub fn get() -> Self {
        Self(stdout())
    }

    pub fn write_bytes(self, bytes: &[u8]) {
        write(self.0, bytes);
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(self.0, s.as_bytes());
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Stderr(isize);

impl Stderr {
    pub fn get() -> Self {
        Self(stderr())
    }

    pub fn write_bytes(self, bytes: &[u8]) {
        write(self.0, bytes);
    }
}

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(self.0, s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    () => {{}};
    ($($tt:tt)*) => {{
        let s = format_args!($($tt)*);
        $crate::print::write_stdout(s);
    }}
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::print::write_stdout(format_args!("\n"))
    }};
    ($($tt:tt)*) => {{
        let s = format_args!($($tt)*);
        $crate::print::write_stdout(format_args!("{s}\n"));
    }}
}

#[macro_export]
macro_rules! eprint {
    () => {{}};
    ($($tt:tt)*) => {{
        let s = format_args!($($tt)*);
        $crate::print::write_stderr(s);
    }}
}

#[macro_export]
macro_rules! eprintln {
    () => {{
        $crate::print::write_stderr(format_args!("\n"))
    }};
    ($($tt:tt)*) => {{
        let s = format_args!($($tt)*);
        $crate::print::write_stderr(format_args!("{s}\n"));
    }}
}