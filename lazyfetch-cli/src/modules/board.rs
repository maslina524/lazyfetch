use proc::{Docs, Module};

use crate::{
    detect::board, impl_display_for_module, modules::Module, str::SmolStr, sync::OnceLock,
};

static BOARD: OnceLock<Board> = OnceLock::new();

#[derive(Default, Debug, Docs, Module)]
pub struct Board {
    pub name: SmolStr,
    pub vendor: SmolStr,
    pub version: SmolStr,
    pub serial: SmolStr,
}

impl Module for Board {
    fn new() -> Self {
        board::get()
    }

    fn get() -> &'static Self {
        BOARD.get_or_init(Self::new)
    }

    fn key(&self) -> &'static str {
        "Board"
    }

    fn title(&self) -> &'static str {
        "{name} {version}"
    }

    fn string_name(&self) -> &'static str {
        "board"
    }
}

impl_display_for_module!(Board);
