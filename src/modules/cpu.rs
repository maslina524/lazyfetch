use alloc::{borrow::Cow, string::String};
use doc::Docs;

use crate::{
    detect::cpu,
    impl_module,
    formats::{
        Frequency,
        Temperature,
        MicroArch
    },
    impl_display_for_module,
    modules::Module,
    sync::OnceLock
};

static CPU: OnceLock<Cpu> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Cpu {
    #[doc = "Name"]
    pub name: Cow<'static, str>,
    #[doc = "Vendor"]
    pub vendor: String,
    #[doc = "Physical core count"]
    pub cores_physical: usize,
    #[doc = "Logical core count"]
    pub cores_logical: usize,
    #[doc = "Online core count"]
    pub cores_online: usize,
    #[doc = "Base frequency (formatted)"]
    pub freq_base: Frequency,
    #[doc = "Max frequency (formatted)"]
    pub freq_max: Frequency,
    #[doc = "Temperature (not available in windows)"]
    pub temperature: Temperature,
    #[doc = "Logical core count grouped by frequency (not available)"]
    pub core_types: String,
    #[doc = "Package count"]
    pub packages: usize,
    #[doc = "Microarchitecture"]
    pub march: MicroArch,
    #[doc = "NUMA node count"]
    pub numa_nodes: usize,
    #[doc = "Code name, like \"Raptor Lake\""]
    pub code_name: &'static str,
    #[doc = "Technology"]
    pub technology: &'static str
}

impl Module for Cpu {
    fn new() -> Self {
        cpu::get()
    }

    fn get() -> &'static Self {
        CPU.get_or_init(|| {
            Self::new()
        })
    }

    fn key(&self) -> &'static str {
        "CPU"
    }

    fn title(&self) -> &'static str {
        if self.core_types.is_empty() {
            "{name} @ {freq-base}"
        } else {
            "{name} ({core-types}) @ {freq-base}"
        }
    }

    fn string_name(&self) -> &'static str {
        "cpu"
    }
    
    impl_module!(
        name, vendor, cores_physical, cores_logical,
        cores_online, freq_base, freq_max, temperature,
        core_types, packages, march, numa_nodes,
        code_name, technology
    );
}

impl_display_for_module!(Cpu);