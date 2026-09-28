use core::{
    fmt::{Debug, Display}, 
    marker::PhantomData, 
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::{
    borrow::ToOwned, 
    boxed::Box, 
    collections::BTreeMap, 
    string::String
};

use crate::{
    format,
    lua::{AsLua, LuaType},
    sync::Mutex,
    warning,
};

pub struct CacheEntry {}

static LOADED: AtomicBool = AtomicBool::new(false);
static EDITED: AtomicBool = AtomicBool::new(false);

static ENTRIES: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

fn load_entries() {
    if LOADED.load(Ordering::Relaxed) {
        LOADED.store(true, Ordering::Relaxed);

    }
}

pub struct SessionCached<T: AsCached> {
    name: Box<str>,
    func: Box<dyn Fn() -> T + Send + Sync>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: AsCached + Clone + Send + Sync + 'static> SessionCached<T> {
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

    fn get_value(&self) -> Option<T> {
        load_entries();
        None
    }
    const fn save_value(&self, _: &T) {}
}

impl<T: Display + AsCached + Clone + Send + Sync + 'static> Display for SessionCached<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.load())
    }
}

impl<T: Debug + AsCached + Clone + Send + Sync + 'static> Debug for SessionCached<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.load())
    }
}

impl<T: Display + AsCached + AsLua + Clone + Send + Sync + 'static> AsLua for SessionCached<T> {
    fn as_lua(&self) -> LuaType {
        LuaType::String(format!("{self}"))
    }
    const LUA_TYPE: &'static str = T::LUA_TYPE;
}


pub trait AsCached: Sized {
    fn as_cached(&self) -> String;
    fn from_cached(cache: &str) -> Option<Self>;
}

impl AsCached for String {
    fn as_cached(&self) -> String {
        format!("\"{self}\"")
    }

    fn from_cached(cache: &str) -> Option<Self> {
        if cache.len() < 2 {
            warning!("Incorrect string in cache");
            None
        } else {
            Some(cache[1..cache.len() - 1].to_owned())
        }
    }
}