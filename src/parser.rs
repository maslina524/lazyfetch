use core::fmt::Display;

use alloc::{
    collections::BTreeMap,
    string::String,
    borrow::ToOwned,
    vec::Vec
};

use crate::imp::{
    fs,
    path::Path
};

pub struct LineBased<'lb> {
    inner: BTreeMap<&'lb str, &'lb str>,
    splitter: char
}

impl<'lb> LineBased<'lb> {
    pub const fn new() -> Self {
        Self { inner: BTreeMap::new(), splitter: '=' }
    }

    pub fn parse_os_release() -> Result<Self, fs::ReadError> {
        Self::parse_file("/etc/os-release", '=')
    }

    pub fn parse_cpu_info() -> Result<Self, fs::ReadError> {
        Self::parse_file("/proc/cpuinfo", ':')
    }

    pub fn parse_mem_info() -> Result<Self, fs::ReadError> {
        Self::parse_file("/proc/meminfo", ':')
    }

    pub fn parse_file(path: impl Into<Path>, split: char) -> Result<Self, fs::ReadError> {
        let string = fs::read_to_string(path)?;
        Ok(Self::parse(string, split))
    }

    pub fn parse(s: String, split: char) -> Self {
        let s: &'static str = String::leak(s);
        let mut ret = BTreeMap::new();

        for line in s.lines() {
            let eq_count = line.chars().filter(|&c| c == split).count();
            if eq_count != 1 {
                continue;
            }

            let eq_index = line.find(split).unwrap();
            let k = line[..eq_index].trim();
            let mut v = line[eq_index + 1..].trim();

            if v.starts_with('"') && v.ends_with('"') {
                v = &v[1..v.len() - 1];
            }

            ret.insert(k, v.trim());
        }

        Self { inner: ret, splitter: split }
    }

    pub fn get_default(&self, key: &str, default: &'lb str) -> &'lb str {
        self
            .inner
            .get(key)
            .copied()
            .unwrap_or(default)
    }

    pub fn get(&self, key: &str) -> Option<&'lb str> {
        self
            .inner
            .get(key)
            .map(|s| s.trim())
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Option<&'lb str> {
        let key: &'static str = String::leak(key.to_owned());
        let value: &'static str = String::leak(value.to_owned());
        self.inner.insert(key, value)
    }
}

pub fn parse_range_notation(s: &str, capacity: Option<usize>) -> Vec<usize> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    
    let mut ret = capacity.map_or_else(Vec::new, Vec::with_capacity);

    for part in s.split(',').map(str::trim) {
        if let Some(pos) = part.find('-') {
            if let (Ok(start), Ok(end)) = (
                part[..pos].parse::<usize>(),
                part[pos + 1..].parse::<usize>(),
            ) && start <= end {
                ret.extend(start..=end);
            }
        } else if let Ok(n) = part.parse::<usize>() {
            ret.push(n);
        }
    }

    ret.sort_unstable();
    ret.dedup();
    ret
}

impl Display for LineBased<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let splitter = self.splitter;
        for (k, v) in &self.inner {
            writeln!(f, "{k}{splitter}{v}")?;
        }
        Ok(())
    }
}