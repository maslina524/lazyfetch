use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    sync::atomic::AtomicBool,
};

use alloc::{boxed::Box, vec::Vec};

use crate::{
    format,
    lua::{AsLua, LuaType},
    sync::Mutex,
};

pub struct CacheEntry {}

static LOADED: AtomicBool = AtomicBool::new(false);
static EDITED: AtomicBool = AtomicBool::new(false);

static ENTRIES: Mutex<Vec<CacheEntry>> = Mutex::new(Vec::new());

pub struct SessionCached<T> {
    name: Box<str>,
    func: Box<dyn Fn() -> T + Send + Sync>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Clone + Send + Sync + 'static> SessionCached<T> {
    pub fn new<F>(name: &str, func: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            func: Box::new(func),
            _marker: PhantomData,
        }
    }

    pub fn load(&self) -> T {
        if let Some(v) = self.get_value() {
            v
        } else {
            let v = (self.func)();
            self.save_value(&v);
            v
        }
    }

    const fn get_value(&self) -> Option<T> {
        None
    }
    const fn save_value(&self, _: &T) {}
}

impl<T: Display + Clone + Send + Sync + 'static> Display for SessionCached<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.load())
    }
}

impl<T: Debug + Clone + Send + Sync + 'static> Debug for SessionCached<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.load())
    }
}

impl<T: Display + AsLua + Clone + Send + Sync + 'static> AsLua for SessionCached<T> {
    fn as_lua(&self) -> LuaType {
        LuaType::String(format!("{self}"))
    }
    const LUA_TYPE: &'static str = T::LUA_TYPE;
}
