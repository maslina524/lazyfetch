use core::fmt::{self, Debug, Display};

use alloc::boxed::Box;

use crate::{
    format,
    lua::{AsLua, LuaType},
    sync::OnceLock,
};

pub struct LazyField<T> {
    value: OnceLock<T>,
    func: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Send + Sync + 'static> LazyField<T> {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            value: OnceLock::new(),
            func: Box::new(func),
        }
    }

    pub fn get(&self) -> &T {
        self.value.get_or_init(|| (self.func)())
    }
}

impl<T: Display + Send + Sync + 'static> Display for LazyField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl<T: Debug + Send + Sync + 'static> Debug for LazyField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.get())
    }
}

impl<T: Display + AsLua + Send + Sync + 'static> AsLua for LazyField<T> {
    fn as_lua(&self) -> LuaType {
        LuaType::String(format!("{self}"))
    }
    const LUA_TYPE: &'static str = T::LUA_TYPE;
}
