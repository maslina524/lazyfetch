use core::ops::Range;

use alloc::{string::String, vec::Vec};

#[derive(Clone)]
pub struct SplittedAnsiIter {
    buf: &'static str,
    ranges: Vec<Range<usize>>,
    index: usize,
}

impl SplittedAnsiIter {
    pub const fn empty() -> Self {
        Self {
            buf: "",
            ranges: Vec::new(),
            index: 0,
        }
    }

    pub fn new(s: &str, len: usize) -> Self {
        if s.is_empty() {
            return Self::empty();
        }

        let mut buf = String::with_capacity(s.len() + 32);
        let mut ranges: Vec<Range<usize>> = Vec::with_capacity(8); // Random cap value

        let mut build_len = 0;
        let mut chunk_start = 0;
        let mut current_ansi = String::from("\x1b[0m");
        let mut ansi_is_default = true;

        let mut it = s.chars();
        while let Some(ch) = it.next() {
            if ch == '\x1b' {
                current_ansi.clear();
                current_ansi.push('\x1b');
                buf.push('\x1b');
                for c in it.by_ref() {
                    current_ansi.push(c);
                    buf.push(c);
                    if c == 'm' {
                        break;
                    }
                }
                ansi_is_default = current_ansi == "\x1b[0m";
                continue;
            }

            if ch == '\n' {
                if !ansi_is_default {
                    buf.push_str("\x1b[0m");
                }

                // if build_len > 0 {
                //     ranges.push(chunk_start..buf.len());
                // } else {
                //     buf.truncate(chunk_start);
                // }
                ranges.push(chunk_start..buf.len());
                build_len = 0;
                chunk_start = buf.len();

                if !ansi_is_default {
                    buf.push_str(&current_ansi);
                }
                continue;
            }

            buf.push(ch);
            build_len += 1;

            if build_len >= len {
                if !ansi_is_default {
                    buf.push_str("\x1b[0m");
                }

                ranges.push(chunk_start..buf.len());
                build_len = 0;
                chunk_start = buf.len();

                if !ansi_is_default {
                    buf.push_str(&current_ansi);
                }
            }
        }

        if current_ansi != "\x1b[0m" {
            buf.push_str("\x1b[0m");
        }
        if chunk_start != buf.len() {
            ranges.push(chunk_start..buf.len());
        }

        Self {
            buf: String::leak(buf),
            ranges,
            index: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.ranges.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}

impl Iterator for SplittedAnsiIter {
    type Item = &'static str;

    fn next(&mut self) -> Option<&'static str> {
        let r = self.ranges.get(self.index)?;
        self.index += 1;
        Some(&self.buf[r.clone()])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.ranges.len() - self.index;
        (rem, Some(rem))
    }
}

impl ExactSizeIterator for SplittedAnsiIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newline_test() {
        let string = "\n\nnewline\nstring\n";
        let splitted_iter = SplittedAnsiIter::new(string, usize::MAX);
        let splitted = splitted_iter.collect::<Vec<&str>>();

        assert_eq!(vec!["", "", "newline", "string", ""], splitted);
    }
}
