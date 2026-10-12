use proc::{Docs, Module};

use crate::{imp::env, impl_display_for_module, modules::Module, sync::OnceLock};

static PROCESSES: OnceLock<Processes> = OnceLock::new();

#[derive(Debug, Docs, Module)]
pub struct Processes {
    #[doc = "Process count"]
    pub result: usize,
}

impl Module for Processes {
    fn new() -> Self {
        let result = env::processes_count();

        Self { result }
    }

    fn get() -> &'static Self {
        PROCESSES.get_or_init(Self::new)
    }

    fn key(&self) -> &'static str {
        "Processes"
    }

    fn title(&self) -> &'static str {
        "{result}"
    }

    fn string_name(&self) -> &'static str {
        "processes"
    }
}

impl_display_for_module!(Processes);
