use core::fmt::{Arguments, Write};

use alloc::{
    vec::Vec,
    string::String
};

use crate::sync::Mutex;

const STRING_BASE_CAP: usize = 64;

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