use alloc::string::String;
use doc::Docs;

use crate::{
    detect::memory::MemoryInfo,
    formats::{MemorySize, Percent},
    impl_display_for_module, impl_module,
    modules::Module,
    sync::OnceLock,
};

static MEMORY: OnceLock<Memory> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Memory {
    #[doc = "Used size"]
    pub used: MemorySize,
    #[doc = "Total size"]
    pub total: MemorySize,
    #[doc = "Percentage used (num)"]
    pub percentage: Percent,
    #[doc = "Percentage used (bar)"]
    pub percentage_bar: String,
}

impl Module for Memory {
    fn new() -> Self {
        let mem = MemoryInfo::new();
        let total = mem.total;
        let used = mem.in_use;
        let percent = (used.as_kilobytes() / total.as_kilobytes() * 100.0) as u8;

        Self {
            used,
            total,
            percentage: Percent::new(percent),
            percentage_bar: String::new(),
        }
    }

    fn get() -> &'static Self {
        MEMORY.get_or_init(|| Self::new())
    }

    fn key(&self) -> &'static str {
        "Memory"
    }

    fn title(&self) -> &'static str {
        "{used} / {total} ({percentage})"
    }

    fn string_name(&self) -> &'static str {
        "cpu"
    }

    impl_module!(used, total, percentage, percentage_bar);
}

impl_display_for_module!(Memory);
