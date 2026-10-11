use alloc::{borrow::Cow, collections::BTreeMap, string::String, vec::Vec};

use proc::{Docs, Module};

use crate::{
    detect::disk::get_disks,
    formats::{MemorySize, Percent, Time},
    impl_display_for_module,
    json::Value,
    modules::{self, Module, ModuleFormat},
    sync::OnceLock,
};

static DISK_LIST: OnceLock<DiskList> = OnceLock::new();

pub struct DiskList {
    pub list: Vec<Disk>,
}

impl DiskList {
    pub fn first_owned(self) -> Disk {
        self.list.first().cloned().unwrap_or_default()
    }
}

impl Module for DiskList {
    fn new() -> Self {
        Self { list: get_disks() }
    }

    fn get() -> &'static Self {
        DISK_LIST.get_or_init(Self::new)
    }

    fn title(&self) -> &'static str {
        ""
    }

    fn key(&self) -> &'static str {
        "disk"
    }

    fn string_name(&self) -> &'static str {
        "disk"
    }
}

impl ModuleFormat for DiskList {
    fn format(
        &self,
        key: super::FormatValue,
        format: super::FormatValue,
        map: Option<&BTreeMap<String, Value>>,
    ) -> Option<Cow<'_, str>> {
        let s = self
            .list
            .iter()
            .filter_map(|d| d.format(key, format, map))
            .collect::<Vec<_>>()
            .join("\n");

        Some(modules::expand_env_in_module(s))
    }
}

#[derive(Debug, Clone, Default, Docs, Module)]
pub struct Disk {
    pub size_used: MemorySize,
    pub size_total: MemorySize,
    pub size_percentage: Percent,
    pub files_used: usize,
    pub files_total: usize,
    pub files_percentage: Percent,
    pub is_external: bool,
    pub is_hidden: bool,
    pub filesystem: String,
    pub name: String,
    pub is_readonly: bool,
    pub create_time: Time,
    pub size_percentage_bar: String,
    pub files_percentage_bar: String,
    pub days: u32,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
    pub mountpoint: String,
    pub mount_from: String,
}

#[allow(clippy::unused_self)]
impl Disk {
    const fn title(&self) -> &'static str {
        "{size-used} / {size-total} ({size-percentage}) - {filesystem}"
    }

    const fn key(&self) -> &'static str {
        "Disk ({mountpoint})"
    }
}

impl_display_for_module!(DiskList);
