use core::fmt::{self, Debug, Display, Formatter};

use crate::{
    format,
    lua::{AsLua, LuaType},
    warning,
};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Level {
    _V1 = 1,
    _V2 = 2,
    _V3 = 3,
    _V4 = 4,
}

#[allow(clippy::used_underscore_items)]
impl TryFrom<u8> for Level {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            1 => Ok(Self::_V1),
            2 => Ok(Self::_V2),
            3 => Ok(Self::_V3),
            4 => Ok(Self::_V4),
            _ => {
                warning!("Incorrect march level value: {value}");
                Err(())
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct MicroArch {
    level: Option<Level>,
}

impl MicroArch {
    pub fn new(level: u8) -> Self {
        Self {
            level: Level::try_from(level).ok(),
        }
    }
}

impl Display for MicroArch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(l) = self.level {
            write!(f, "{}-v{}", env!("TARGET_ARCH"), l as u8)?;
        } else {
            write!(f, "{}", env!("TARGET_ARCH"))?;
        }
        Ok(())
    }
}

impl Debug for MicroArch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl AsLua for MicroArch {
    fn as_lua(&self) -> LuaType {
        LuaType::String(format!("{self:?}"))
    }
    const LUA_TYPE: &'static str = "string";
}
