use core::{
    ffi::{CStr, c_char, c_int, c_void},
    ptr,
};

use alloc::{borrow::Cow, collections::BTreeMap, ffi::CString, string::String};

use crate::{
    abort, 
    cfg_if, 
    format, 
    formats::{expand_rust_unicode, snake_to_camel_ascii}, 
    imp::fs::{self, ReadError}, 
    str::ConcatStr, 
    sync::OnceLock, 
    warning,
};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use crate::{
            windows::link::HMODULE,
        };

        type ApiBaseFn = unsafe extern "system" fn() -> isize;
        type LibHandle = HMODULE;
    }
}

#[allow(non_camel_case_types)]
pub type lua_State = *mut c_void;

#[allow(non_camel_case_types)]
pub type luaL_newstate = unsafe extern "C" fn() -> *mut lua_State;
#[allow(non_camel_case_types)]
pub type luaL_openlibs = unsafe extern "C" fn(state: *mut lua_State);
#[allow(non_camel_case_types)]
pub type luaL_openselectedlibs = unsafe extern "C" fn(state: *mut lua_State, mask: c_int);
#[allow(non_camel_case_types)]
pub type luaL_loadstring = unsafe extern "C" fn(state: *mut lua_State, s: *const c_char) -> c_int;
#[allow(non_camel_case_types)]
pub type lua_pcallk = unsafe extern "C" fn(
    state: *mut lua_State,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
    ctx: isize,
    k: Option<unsafe extern "C" fn(*mut lua_State, c_int, isize)>,
) -> c_int;
#[allow(non_camel_case_types)]
pub type lua_close = unsafe extern "C" fn(state: *mut lua_State);
#[allow(non_camel_case_types)]
pub type lua_tolstring =
    unsafe extern "C" fn(state: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;
#[allow(non_camel_case_types)]
pub type lua_settop = unsafe extern "C" fn(state: *mut lua_State, idx: c_int);

unsafe extern "C" {
    unsafe fn luaL_newstate() -> *mut lua_State;
    unsafe fn luaL_loadstring(state: *mut lua_State, s: *const c_char) -> c_int;
    unsafe fn lua_pcallk(
        state: *mut lua_State,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: isize,
        k: Option<unsafe extern "C" fn(*mut lua_State, c_int, isize)>,
    ) -> c_int;
    unsafe fn lua_close(state: *mut lua_State);
    unsafe fn lua_tolstring(state: *mut lua_State, idx: c_int, len: *mut usize) -> *const c_char;
    unsafe fn lua_settop(state: *mut lua_State, idx: c_int);

    #[cfg(lua5_4)]
    unsafe fn luaL_openlibs(state: *mut lua_State);
    #[cfg(lua5_5)]
    unsafe fn luaL_openselectedlibs(state: *mut lua_State, mask: c_int);
}

static LUA: OnceLock<LuaLib> = OnceLock::new();

pub enum LuaType {
    String(String),
    Number(f64),
    Boolean(bool),
}

pub trait AsLua {
    const LUA_TYPE: &'static str;
    fn as_lua(&self) -> LuaType;
}

macro_rules! impl_as_lua_debug_string {
    ($($typ:ty),* $(,)?) => {
        $(
            impl AsLua for $typ {
                fn as_lua(&self) -> LuaType {
                    LuaType::String(crate::format!("{self:?}"))
                }

                const LUA_TYPE: &'static str = "string";
            }
        )*
    }
}

macro_rules! impl_as_lua_to_string {
    ($($typ:ty),* $(,)?) => {
        $(
            impl AsLua for $typ {
                fn as_lua(&self) -> LuaType {
                    LuaType::String(alloc::string::ToString::to_string(self))
                }

                const LUA_TYPE: &'static str = "string";
            }
        )*
    }
}

macro_rules! impl_as_lua_into_f64 {
    ($($typ:ty),* $(,)?) => {
        $(
            impl AsLua for $typ {
                fn as_lua(&self) -> LuaType {
                    LuaType::Number(f64::from(*self))
                }

                const LUA_TYPE: &'static str = "number";
            }
        )*
    }
}

macro_rules! impl_as_lua_as_f64 {
    ($($typ:ty),* $(,)?) => {
        $(
            impl AsLua for $typ {
                fn as_lua(&self) -> LuaType {
                    #[allow(clippy::cast_precision_loss)]
                    LuaType::Number(*self as f64)
                }

                const LUA_TYPE: &'static str = "number";
            }
        )*
    }
}

impl_as_lua_debug_string!(
    String,
    &str,
    str,
    char,
    crate::detect::gpu::GpuType,
    crate::imp::path::Path,
    crate::str::SmolStr,
    alloc::borrow::Cow<'static, str>
);
impl_as_lua_to_string!(
    crate::detect::datetime::AmPm
);
impl_as_lua_into_f64!(
    crate::formats::Temperature,
    crate::formats::Percent,
    crate::formats::MemorySize,
    crate::formats::Frequency,
    crate::formats::Time
);
impl_as_lua_as_f64!(
    usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, f32, f64,
);
impl AsLua for bool {
    fn as_lua(&self) -> LuaType {
        #[allow(clippy::cast_precision_loss)]
        LuaType::Boolean(*self)
    }
    const LUA_TYPE: &'static str = "boolean";
}
impl<const N: usize> AsLua for ConcatStr<N> {
    fn as_lua(&self) -> LuaType {
        #[allow(clippy::cast_precision_loss)]
        LuaType::String(format!("{self:?}"))
    }
    const LUA_TYPE: &'static str = "string";
}

impl core::fmt::Debug for LuaType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::String(v) => write!(f, "{v}"),
            Self::Number(v) => write!(f, "{v}"),
            Self::Boolean(v) => write!(f, "{v}"),
        }
    }
}

pub fn open_lua_file(code: &str) -> Cow<'_, str> {
    match fs::read_to_string(code.trim()) {
        Ok(c) => Cow::Owned(c),
        Err(e) => match e {
            ReadError::Code(e) => {
                if e.is_file_not_found() {
                    Cow::Borrowed(code)
                } else {
                    warning!("Failed to open file: {e}");
                    Cow::Owned(String::new())
                }
            }
            ReadError::Utf8(e) => {
                warning!("Failed to open the file as Utf8: {e}");
                Cow::Owned(String::new())
            }
        },
    }
}

pub struct LuaLib {
    new_state: luaL_newstate,
    load_string: luaL_loadstring,
    pcall: lua_pcallk,
    close: lua_close,
    to_lstring: lua_tolstring,
    #[cfg(lua5_4)]
    open_libs: luaL_openlibs,
    #[cfg(lua5_5)]
    open_selected_libs: luaL_openselectedlibs,
}

// SAFETY: The structure is not thread-safe; however we never mutate its fields,
// and each call creates a fresh Lua state, so concurrent use from different threads
// is safe as long as the underlying Lua library is thread‑aware (it is).
unsafe impl Sync for LuaLib {}

impl LuaLib {
    pub fn get() -> &'static Self {
        LUA.get_or_init(Self::new_static)
    }

    pub fn new_static() -> Self {
        Self {
            new_state: luaL_newstate,
            load_string: luaL_loadstring,
            pcall: lua_pcallk,
            close: lua_close,
            to_lstring: lua_tolstring,
            #[cfg(lua5_4)]
            open_libs: luaL_openlibs,
            #[cfg(lua5_5)]
            open_selected_libs: luaL_openselectedlibs,
        }
    }

    pub fn exec(&self, code: &str, vars: BTreeMap<String, LuaType>) -> String {
        let code = code.trim();
        let mut ret = String::with_capacity(128 + code.len() + vars.len() * 16);

        ret.push_str("local module_data = {\n");
        for (k, v) in vars {
            ret.push_str("    ");
            ret.push_str(&snake_to_camel_ascii(&k));
            ret.push_str(" = ");
            ret.push_str(&format!("{:?}", v));
            ret.push_str(",\n");
        }

        ret.push_str("}\n\nlocal user_code = function(...)\n    ");
        ret.push_str(code);
        ret.push_str("\nend\n\nreturn user_code(module_data)");
        ret = expand_rust_unicode(&ret);

        // crate::println!("{ret}");
        self.exec_without_vars(&ret)
    }

    pub fn exec_without_vars(&self, code: &str) -> String {
        // SAFETY: `luaL_newstate` returns a valid state or NULL
        let state = unsafe { (self.new_state)() };
        if state.is_null() {
            abort!("Failed to create new Lua state");
        }

        #[cfg(lua5_4)]
        // SAFETY: Completely safe
        unsafe {
            (self.open_libs)(state)
        };

        #[cfg(lua5_5)]
        // SAFETY: `luaL_openselectedlibs` takes a valid state and opens all
        // standard libraries (mask = -1 means all)
        unsafe {
            (self.open_selected_libs)(state, -1);
        }

        let c_code = CString::new(code).expect("Lua code contains NUL bytes");

        // SAFETY: `luaL_loadstring` compiles the chunk and leaves the function
        // on top of the stack. Returns 0 on success, non‑zero on error
        let load_err = unsafe { (self.load_string)(state, c_code.as_ptr()) };

        if load_err != 0 {
            // SAFETY: `lua_tolstring` returns a pointer to the error string
            let err_ptr = unsafe { (self.to_lstring)(state, -1, ptr::null_mut()) };
            let err_msg = if err_ptr.is_null() {
                String::from("unknown error (nil)")
            } else {
                // SAFETY: Lua guarantees a valid null‑terminated string
                unsafe { CStr::from_ptr(err_ptr) }
                    .to_string_lossy()
                    .into_owned()
            };
            // SAFETY: Close the state to free resources
            unsafe { (self.close)(state) };
            abort!("Lua compilation error: {}", err_msg);
        }

        // SAFETY: Call the compiled function with 0 arguments and expect 1 result
        // The last argument is a continuation function pointer (NULL)
        let pcall_err = unsafe { (self.pcall)(state, 0, 1, 0, 0, None) };

        if pcall_err != 0 {
            // SAFETY: `lua_tolstring` returns a pointer to the error string
            let err_ptr = unsafe { (self.to_lstring)(state, -1, ptr::null_mut()) };
            let err_msg = if err_ptr.is_null() {
                String::from("unknown error (nil)")
            } else {
                // SAFETY: Lua guarantees a valid null‑terminated string
                unsafe { CStr::from_ptr(err_ptr) }
                    .to_string_lossy()
                    .into_owned()
            };
            // SAFETY: Close the state to free resources
            unsafe { (self.close)(state) };
            abort!("Lua runtime error: {}", err_msg);
        }

        // SAFETY: `lua_tolstring` returns a pointer to the string value
        let result_ptr = unsafe { (self.to_lstring)(state, -1, ptr::null_mut()) };

        let result_string = if result_ptr.is_null() {
            String::from("nil")
        } else {
            // SAFETY: Lua guarantees a valid null‑terminated string
            unsafe { CStr::from_ptr(result_ptr) }
                .to_string_lossy()
                .into_owned()
        };

        // SAFETY: Close the state to free resources
        unsafe { (self.close)(state) };

        result_string
    }
}

#[cfg(test)]
mod tests {
    use crate::lua::LuaLib;

    #[test]
    fn exec_lua_code_test() {
        let lib = LuaLib::get();
        let result = lib.exec_without_vars("return 2 + 2;");
        assert_eq!(result, "4");
    }
}
